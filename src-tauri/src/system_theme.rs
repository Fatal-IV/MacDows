//! Matches the Windows surfaces MacDows cannot draw itself (Start, Search, quick settings) to the app theme.
//!
//! Those menus belong to Windows, but they take their light or dark look and their tint from the personalization
//! settings. While MacDows replaces the taskbar it sets those settings from its own theme (a dark or light system
//! theme, and a tint derived from the MacDows surface color) and puts the originals back when the taskbar comes
//! back, whichever way MacDows ends (the exit hook, the guard process, `--restore-taskbar`).
//!
//! Two levels. The safe one only follows MacDows's light or dark mode with the *system* light/dark switch (never the
//! one for apps, so windows keep the user's choice). The tint level also paints Start and the taskbar in the MacDows
//! surface color, which Windows only does through the system accent color, so while MacDows runs the accent color of
//! every app is that surface color too (the title-bar switch is left alone). It is off unless the user turns it on.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE};
use winreg::{RegKey, RegValue};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};

const PERSONALIZE: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const ACCENT: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent";
const DWM: &str = r"Software\Microsoft\Windows\DWM";

/// Brightness of the eight accent shades, lightest to darkest. Windows uses index 3 as the accent itself and
/// index 4 as the Start color.
const SHADES: [f64; 8] = [1.5, 1.25, 1.1, 1.0, 0.9, 0.8, 0.65, 0.5];

static ENABLED: AtomicBool = AtomicBool::new(false);
static LAST: Mutex<Option<(bool, String, bool)>> = Mutex::new(None);
/// The latest theme the notch asked for, kept even while the takeover is not active yet (the notch applies its theme
/// before the taskbar is hidden).
static REQUESTED: Mutex<Option<(bool, String, bool)>> = Mutex::new(None);

// ---------------------------------------------------------------------------------------------
// Pure logic (tested)
// ---------------------------------------------------------------------------------------------

pub fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let h = s.trim().trim_start_matches('#');
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some(((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

fn shade((r, g, b): (u8, u8, u8), factor: f64) -> (u8, u8, u8) {
    let f = |c: u8| (c as f64 * factor).round().clamp(0.0, 255.0) as u8;
    (f(r), f(g), f(b))
}

/// Windows stores colors as 0xAABBGGRR.
fn abgr((r, g, b): (u8, u8, u8)) -> u32 {
    0xFF00_0000 | (b as u32) << 16 | (g as u32) << 8 | r as u32
}

#[derive(Debug, PartialEq)]
pub struct Tint {
    pub prevalence: u32,
    /// The color of the taskbar surface (`AccentColorMenu`).
    pub accent_menu: u32,
    /// The color of Start and Search (`StartColorMenu`).
    pub start_menu: u32,
    /// The two palette entries Start reads (see `palette_with_start`), darkest last.
    pub start_rgb: [(u8, u8, u8); 2],
}

#[derive(Debug, PartialEq)]
pub struct Plan {
    pub system_light: u32,
    /// Only when the user asked for the Start menu to take the MacDows color.
    pub tint: Option<Tint>,
}

/// What to write for a light or dark MacDows with the given surface color. The window title bar color and the app
/// theme are never changed.
pub fn plan(light: bool, surface: (u8, u8, u8), tint: bool) -> Plan {
    // Only the two menu colors are written. The accent palette and the window colors stay the user's own, so
    // apps, title bars and the quick settings keep their accent color.
    let tint = (tint && !light).then(|| Tint {
        // Windows tints Start and the taskbar with these colors only in the dark system theme.
        prevalence: 1,
        accent_menu: abgr(shade(surface, SHADES[3])),
        start_menu: abgr(shade(surface, SHADES[4])),
        start_rgb: [shade(surface, SHADES[4]), shade(surface, SHADES[5])],
    });
    Plan { system_light: light as u32, tint }
}

/// The accent palette with only the entries Start reads painted: the fifth of eight (bytes 16 to 18) on older
/// Windows 11 builds and the sixth (bytes 20 to 22) on current ones (checked on build 26300, where painting the fifth
/// did nothing and the sixth made Start black). The other six entries, which apps and windows read, stay as they are.
/// `None` when the palette is not the 32 bytes Windows writes.
pub fn palette_with_start(palette: &[u8], rgb: [(u8, u8, u8); 2]) -> Option<Vec<u8>> {
    if palette.len() != 32 {
        return None;
    }
    let mut out = palette.to_vec();
    for (entry, (r, g, b)) in [4usize, 5].into_iter().zip(rgb) {
        out[entry * 4] = r;
        out[entry * 4 + 1] = g;
        out[entry * 4 + 2] = b;
    }
    Some(out)
}

// ---------------------------------------------------------------------------------------------
// Backup
// ---------------------------------------------------------------------------------------------

/// The values as they were, `None` where the value did not exist.
#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Clone)]
struct Backup {
    system_light: Option<u32>,
    prevalence: Option<u32>,
    palette: Option<Vec<u8>>,
    accent_menu: Option<u32>,
    start_menu: Option<u32>,
    dwm_accent: Option<u32>,
    dwm_colorization: Option<u32>,
    dwm_prevalence: Option<u32>,
}

fn backup_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("MacDows").join("system-theme-backup.json")
}

fn open(path: &str) -> Option<RegKey> {
    RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(path, KEY_READ | KEY_SET_VALUE).ok()
}

fn read_u32(key: &Option<RegKey>, name: &str) -> Option<u32> {
    key.as_ref()?.get_value::<u32, _>(name).ok()
}

fn read_backup() -> Backup {
    let (pz, ac, dw) = (open(PERSONALIZE), open(ACCENT), open(DWM));
    Backup {
        system_light: read_u32(&pz, "SystemUsesLightTheme"),
        prevalence: read_u32(&pz, "ColorPrevalence"),
        palette: ac.as_ref().and_then(|k| k.get_raw_value("AccentPalette").ok()).map(|v| v.bytes),
        accent_menu: read_u32(&ac, "AccentColorMenu"),
        start_menu: read_u32(&ac, "StartColorMenu"),
        dwm_accent: read_u32(&dw, "AccentColor"),
        dwm_colorization: read_u32(&dw, "ColorizationColor"),
        dwm_prevalence: read_u32(&dw, "ColorPrevalence"),
    }
}

fn write_u32(key: &Option<RegKey>, name: &str, value: Option<u32>) {
    let Some(k) = key else { return };
    match value {
        Some(v) => {
            let _ = k.set_value(name, &v);
        }
        None => {
            let _ = k.delete_value(name);
        }
    }
}

fn write_palette(key: &Option<RegKey>, bytes: Option<&[u8]>) {
    let Some(k) = key else { return };
    match bytes {
        Some(b) => {
            let _ = k.set_raw_value("AccentPalette", &RegValue { bytes: b.to_vec(), vtype: winreg::enums::REG_BINARY });
        }
        None => {
            let _ = k.delete_value("AccentPalette");
        }
    }
}

fn write_all(b: &Backup) {
    let (pz, ac, dw) = (open(PERSONALIZE), open(ACCENT), open(DWM));
    write_u32(&pz, "SystemUsesLightTheme", b.system_light);
    write_u32(&pz, "ColorPrevalence", b.prevalence);
    write_palette(&ac, b.palette.as_deref());
    write_u32(&ac, "AccentColorMenu", b.accent_menu);
    write_u32(&ac, "StartColorMenu", b.start_menu);
    write_u32(&dw, "AccentColor", b.dwm_accent);
    write_u32(&dw, "ColorizationColor", b.dwm_colorization);
    write_u32(&dw, "ColorPrevalence", b.dwm_prevalence);
}

/// Tells Windows and its shell components to read the colors again.
fn broadcast() {
    let text: Vec<u16> = "ImmersiveColorSet".encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, WPARAM(0), LPARAM(text.as_ptr() as isize), SMTO_ABORTIFHUNG, 3000, None);
    }
}

// ---------------------------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------------------------

/// Only acts while MacDows has taken over the taskbar.
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::SeqCst);
    if on {
        let requested = REQUESTED.lock().unwrap_or_else(|e| e.into_inner()).clone();
        if let Some((light, surface, tint)) = requested {
            apply(light, &surface, tint);
        }
    } else {
        restore();
    }
}

/// Sets the Windows personalization colors for the given MacDows theme.
pub fn apply(light: bool, surface_hex: &str, tint: bool) {
    *REQUESTED.lock().unwrap_or_else(|e| e.into_inner()) = Some((light, surface_hex.to_string(), tint));
    if !ENABLED.load(Ordering::SeqCst) {
        return;
    }
    let Some(surface) = parse_hex(surface_hex) else { return };
    {
        let mut last = LAST.lock().unwrap_or_else(|e| e.into_inner());
        if last.as_ref().is_some_and(|(l, h, t)| *l == light && *t == tint && h.eq_ignore_ascii_case(surface_hex)) {
            return;
        }
        *last = Some((light, surface_hex.to_string(), tint));
    }
    // The first time only: remember what the user had, so it can always be put back. An existing backup is kept,
    // because after a crash the current values are already ours.
    let path = backup_path();
    if !path.exists() {
        if let Ok(json) = serde_json::to_string_pretty(&read_backup()) {
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            if std::fs::write(&path, json).is_err() {
                // Without a backup nothing is changed.
                return;
            }
        }
    }
    let p = plan(light, surface, tint);
    let (pz, ac, dw) = (open(PERSONALIZE), open(ACCENT), open(DWM));
    write_u32(&pz, "SystemUsesLightTheme", Some(p.system_light));
    match &p.tint {
        Some(t) => {
            write_u32(&pz, "ColorPrevalence", Some(t.prevalence));
            write_u32(&ac, "AccentColorMenu", Some(t.accent_menu));
            write_u32(&ac, "StartColorMenu", Some(t.start_menu));
            if let Some(b) = read_backup_file() {
                // The current Start menu takes its color from entries of the accent palette (not from
                // `StartColorMenu`), so those two are painted. The rest of the palette and the window colors stay
                // the user's own, so apps keep their accent color.
                let palette = b.palette.as_deref().and_then(|p| palette_with_start(p, t.start_rgb)).or_else(|| b.palette.clone());
                write_palette(&ac, palette.as_deref());
                write_u32(&dw, "AccentColor", b.dwm_accent);
                write_u32(&dw, "ColorizationColor", b.dwm_colorization);
            }
        }
        None => {
            // Back to the user's own accent and tint settings.
            if let Some(b) = read_backup_file() {
                write_u32(&pz, "ColorPrevalence", b.prevalence);
                write_palette(&ac, b.palette.as_deref());
                write_u32(&ac, "AccentColorMenu", b.accent_menu);
                write_u32(&ac, "StartColorMenu", b.start_menu);
                write_u32(&dw, "AccentColor", b.dwm_accent);
                write_u32(&dw, "ColorizationColor", b.dwm_colorization);
            }
        }
    }
    broadcast();
    reload_start();
}

/// Start reads its colors when it starts: end its process so Windows brings it back with the new ones the next time
/// Start is opened.
fn reload_start() {
    use std::os::windows::process::CommandExt;
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_SHUTTINGDOWN};
    // While Windows is signing out or shutting down no new process can start (taskkill fails with 0xc0000142 and
    // Windows shows an error box that can cancel the shutdown), and Start is going away anyway.
    if unsafe { GetSystemMetrics(SM_SHUTTINGDOWN) } != 0 {
        return;
    }
    let system32 = PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into())).join("System32");
    // Search (SearchHost) reads the colors the same way, so it is restarted with Start.
    let _ = std::process::Command::new(system32.join("taskkill.exe"))
        .args(["/F", "/IM", "StartMenuExperienceHost.exe", "/IM", "SearchHost.exe"])
        .creation_flags(0x0800_0000)
        .output();
}

/// Puts the user's own personalization colors back. Safe to call any number of times.
pub fn restore() {
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) = None;
    let path = backup_path();
    let Some(backup) = read_backup_file() else { return };
    write_all(&backup);
    broadcast();
    reload_start();
    let _ = std::fs::remove_file(path);
}

fn read_backup_file() -> Option<Backup> {
    std::fs::read_to_string(backup_path()).ok().and_then(|s| serde_json::from_str(&s).ok())
}

/// Called by the notch window whenever its theme is applied.
#[tauri::command]
pub async fn sync_system_theme(light: bool, surface: String, tint: bool) {
    let _ = tauri::async_runtime::spawn_blocking(move || apply(light, &surface, tint)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_two_start_entries_of_the_palette_are_painted() {
        let original: Vec<u8> = (0..32).collect();
        let painted = palette_with_start(&original, [(1, 2, 3), (4, 5, 6)]).unwrap();
        assert_eq!(&painted[16..19], &[1, 2, 3]);
        assert_eq!(&painted[20..23], &[4, 5, 6]);
        // Every other byte, including the fourth byte of each painted entry, is the user's own.
        for i in (0..16).chain([19]).chain(23..32) {
            assert_eq!(painted[i], original[i], "byte {i}");
        }
    }

    #[test]
    fn a_palette_of_the_wrong_size_is_left_alone() {
        assert_eq!(palette_with_start(&[0; 31], [(1, 2, 3), (4, 5, 6)]), None);
        assert_eq!(palette_with_start(&[], [(1, 2, 3), (4, 5, 6)]), None);
    }

    #[test]
    fn the_start_palette_colors_are_the_two_darkest_start_shades() {
        let t = plan(false, (0x20, 0x20, 0x24), true).tint.unwrap();
        assert_eq!(t.start_rgb[0], shade((0x20, 0x20, 0x24), 0.9));
        assert_eq!(t.start_rgb[1], shade((0x20, 0x20, 0x24), 0.8));
        assert_eq!(abgr(t.start_rgb[0]), t.start_menu);
    }

    #[test]
    fn hex_colors_are_parsed_with_or_without_the_hash() {
        assert_eq!(parse_hex("#202024"), Some((0x20, 0x20, 0x24)));
        assert_eq!(parse_hex("f5f5f7"), Some((0xf5, 0xf5, 0xf7)));
        assert_eq!(parse_hex("#fff"), None);
        assert_eq!(parse_hex("zzzzzz"), None);
        assert_eq!(parse_hex(""), None);
    }

    #[test]
    fn windows_colors_are_stored_as_abgr() {
        assert_eq!(abgr((0x31, 0x7C, 0x93)), 0xFF93_7C31);
        assert_eq!(abgr((0, 0, 0)), 0xFF00_0000);
    }

    #[test]
    fn the_safe_level_only_follows_light_and_dark() {
        assert_eq!(plan(false, (0x20, 0x20, 0x24), false), Plan { system_light: 0, tint: None });
        assert_eq!(plan(true, (0xF5, 0xF5, 0xF7), false), Plan { system_light: 1, tint: None });
    }

    #[test]
    fn the_tint_level_paints_start_in_the_surface_color_in_dark_mode() {
        let p = plan(false, (0x20, 0x20, 0x24), true);
        assert_eq!(p.system_light, 0);
        let t = p.tint.expect("tint");
        assert_eq!(t.prevalence, 1);
        // The taskbar gets the surface color itself; Start, a little darker.
        assert_eq!(t.accent_menu, abgr((0x20, 0x20, 0x24)));
        assert_eq!(t.start_menu, abgr(shade((0x20, 0x20, 0x24), 0.9)));
    }

    #[test]
    fn a_light_theme_is_never_tinted() {
        let p = plan(true, (0xF5, 0xF5, 0xF7), true);
        assert_eq!(p.system_light, 1);
        assert_eq!(p.tint, None);
    }

    #[test]
    fn the_tint_never_touches_the_accent_color_of_apps() {
        // The Tint holds nothing but the three menu values; the palette and the window colors are not part of it.
        let t = plan(false, (0xFF, 0xFF, 0xFF), true).tint.unwrap();
        assert_eq!(t.accent_menu, abgr((255, 255, 255)));
        assert_eq!(t.start_menu & 0xFF00_0000, 0xFF00_0000);
    }

    #[test]
    fn a_backup_survives_a_round_trip_with_missing_values() {
        let b = Backup { system_light: Some(0), prevalence: None, palette: Some(vec![1, 2, 3]), ..Default::default() };
        let json = serde_json::to_string(&b).unwrap();
        assert_eq!(serde_json::from_str::<Backup>(&json).unwrap(), b);
    }

    #[test]
    fn nothing_is_written_while_the_taskbar_is_not_replaced() {
        // Disabled is the default. (`set_enabled(false)` is not called here: it restores, and a real backup may
        // exist while MacDows runs.)
        apply(false, "#101010", true);
        assert!(LAST.lock().unwrap().is_none());
        // ...but the request is remembered for when it is.
        assert_eq!(REQUESTED.lock().unwrap().clone(), Some((false, "#101010".to_string(), true)));
    }
}
