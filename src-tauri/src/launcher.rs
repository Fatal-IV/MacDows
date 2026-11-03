//! Data and actions behind MacDows's own Start menu: the installed apps, pinned and recently used apps,
//! shortcuts to well-known places, and the power buttons.
//!
//! The app list is Windows's own "Apps" shell folder (the same list Start uses), so classic programs and Store
//! apps both appear, with their real icons, and launching goes through the shell.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use windows::Win32::Foundation::SIZE;
use windows::Win32::Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::Win32::UI::Shell::{
    BHID_EnumItems, FOLDERID_AppsFolder, IEnumShellItems, IShellItem, IShellItemImageFactory, SHGetKnownFolderItem, ShellExecuteW,
    KF_FLAG_DEFAULT, SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVEPARSING, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY,
};
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{Interface, PCWSTR};

const ICON_SIZE: i32 = 64;
const APPS_MAX_AGE: Duration = Duration::from_secs(300);
const RECENT_COUNT: usize = 6;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct AppEntry {
    /// The shell's parsing name (an AppUserModelID or a path): what `shell:AppsFolder\<id>` launches.
    pub id: String,
    pub name: String,
    pub icon_png_base64: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
struct Usage {
    count: u32,
    last: u64,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
struct Store {
    pinned: Vec<String>,
    usage: HashMap<String, Usage>,
}

#[derive(Serialize, Clone, Debug)]
pub struct StartState {
    pub pinned: Vec<String>,
    pub recent: Vec<String>,
    pub user: String,
}

static APPS: Mutex<Option<(Instant, Vec<AppEntry>)>> = Mutex::new(None);
static STORE: Mutex<Option<Store>> = Mutex::new(None);

// ---------------------------------------------------------------------------------------------
// Pure logic (tested)
// ---------------------------------------------------------------------------------------------

/// Most used first; a recent launch breaks ties. Apps that are no longer installed are skipped.
fn top_recent(usage: &HashMap<String, Usage>, known: &dyn Fn(&str) -> bool, n: usize) -> Vec<String> {
    let mut entries: Vec<(&String, &Usage)> = usage.iter().filter(|(id, _)| known(id)).collect();
    entries.sort_by(|a, b| b.1.last.cmp(&a.1.last).then(b.1.count.cmp(&a.1.count)).then(a.0.cmp(b.0)));
    entries.into_iter().take(n).map(|(id, _)| id.clone()).collect()
}

fn toggle(pinned: &mut Vec<String>, id: &str) {
    if let Some(pos) = pinned.iter().position(|p| p == id) {
        pinned.remove(pos);
    } else {
        pinned.push(id.to_string());
    }
}

/// App names that should start pinned the first time Start is opened (Turkish and English names).
const DEFAULT_PINS: &[&str] = &[
    "microsoft edge", "dosya gezgini", "file explorer", "ayarlar", "settings", "terminal", "windows terminal", "not defteri",
    "notepad", "hesap makinesi", "calculator", "microsoft store", "fotoğraflar", "photos",
];

fn default_pins(apps: &[AppEntry]) -> Vec<String> {
    let mut out = Vec::new();
    for wanted in DEFAULT_PINS {
        if let Some(app) = apps.iter().find(|a| a.name.to_lowercase() == *wanted) {
            if !out.contains(&app.id) {
                out.push(app.id.clone());
            }
        }
    }
    out
}

/// Uninstall shortcuts ("Foo Uninstall", "Foo'yu Kaldır") also live in the Apps folder; they do not belong in Start.
fn is_uninstaller(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    n.starts_with("uninstall ") || n.ends_with(" uninstall") || n.ends_with(" kaldır") || n.ends_with(" kaldırma") || n.ends_with(" kaldırın")
}

/// What a power button runs: an absolute path under System32 (never searched on PATH) and its arguments.
pub fn power_command(kind: &str) -> Option<(&'static str, &'static [&'static str])> {
    match kind {
        "shutdown" => Some(("shutdown.exe", &["/s", "/t", "0"])),
        "restart" => Some(("shutdown.exe", &["/r", "/t", "0"])),
        "signout" => Some(("shutdown.exe", &["/l"])),
        "lock" => Some(("rundll32.exe", &["user32.dll,LockWorkStation"])),
        "sleep" => Some(("rundll32.exe", &["powrprof.dll,SetSuspendState", "0,1,0"])),
        _ => None,
    }
}

/// Where each shortcut button goes.
pub fn location_target(kind: &str) -> Option<&'static str> {
    match kind {
        "settings" => Some("ms-settings:"),
        "explorer" => Some("explorer.exe"),
        "documents" => Some("shell:Personal"),
        "downloads" => Some("shell:Downloads"),
        "pictures" => Some("shell:My Pictures"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// Storage
// ---------------------------------------------------------------------------------------------

fn store_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("MacDows").join("launcher.json")
}

fn with_store<R>(f: impl FnOnce(&mut Store) -> R) -> R {
    let mut guard = STORE.lock().unwrap();
    let store = guard.get_or_insert_with(|| std::fs::read_to_string(store_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default());
    let result = f(store);
    if let Ok(json) = serde_json::to_string_pretty(&*store) {
        let path = store_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, json);
    }
    result
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

// ---------------------------------------------------------------------------------------------
// Installed apps
// ---------------------------------------------------------------------------------------------

/// Converts a shell bitmap (32-bit, premultiplied alpha) into a PNG data of straight RGBA.
fn bitmap_png(hbm: HBITMAP) -> Option<String> {
    unsafe {
        let mut bm = BITMAP::default();
        if GetObjectW(hbm.into(), std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut _)) == 0 {
            return None;
        }
        let (w, h) = (bm.bmWidth, bm.bmHeight);
        if w <= 0 || h <= 0 {
            return None;
        }
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut px = vec![0u8; (w * h * 4) as usize];
        let dc = GetDC(None);
        let rows = GetDIBits(dc, hbm, 0, h as u32, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
        ReleaseDC(None, dc);
        if rows == 0 {
            return None;
        }
        for p in px.chunks_exact_mut(4) {
            let a = p[3] as u32;
            let (b, g, r) = (p[0], p[1], p[2]);
            if a > 0 && a < 255 {
                p[0] = (r as u32 * 255 / a).min(255) as u8;
                p[1] = (g as u32 * 255 / a).min(255) as u8;
                p[2] = (b as u32 * 255 / a).min(255) as u8;
            } else {
                p[0] = r;
                p[1] = g;
                p[2] = b;
            }
        }
        let mut out = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header().ok()?.write_image_data(&px).ok()?;
        }
        use base64::Engine;
        Some(base64::engine::general_purpose::STANDARD.encode(out))
    }
}

fn read_apps() -> Vec<AppEntry> {
    let mut apps = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let folder: windows::core::Result<IShellItem> = SHGetKnownFolderItem(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None);
        if let Ok(folder) = folder
            && let Ok(items) = folder.BindToHandler::<_, IEnumShellItems>(None, &BHID_EnumItems)
        {
            loop {
                let mut slot: [Option<IShellItem>; 1] = [None];
                let mut fetched = 0u32;
                if items.Next(&mut slot, Some(&mut fetched)).is_err() || fetched == 0 {
                    break;
                }
                let Some(item) = slot[0].take() else { break };
                let name = item.GetDisplayName(SIGDN_NORMALDISPLAY).ok().and_then(|p| p.to_string().ok()).unwrap_or_default();
                let id = item.GetDisplayName(SIGDN_PARENTRELATIVEPARSING).ok().and_then(|p| p.to_string().ok()).unwrap_or_default();
                if name.trim().is_empty() || id.is_empty() || is_uninstaller(&name) {
                    continue;
                }
                let icon = item.cast::<IShellItemImageFactory>().ok().and_then(|f| {
                    let hbm = f.GetImage(SIZE { cx: ICON_SIZE, cy: ICON_SIZE }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK).ok()?;
                    let png = bitmap_png(hbm);
                    let _ = DeleteObject(hbm.into());
                    png
                });
                apps.push(AppEntry { id, name, icon_png_base64: icon });
            }
        }
        CoUninitialize();
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.id == b.id);
    apps
}

/// The installed apps. Read once, then refreshed in the background when older than five minutes.
pub fn apps() -> Vec<AppEntry> {
    let cached = APPS.lock().unwrap().clone();
    match cached {
        Some((at, list)) => {
            if at.elapsed() > APPS_MAX_AGE {
                std::thread::spawn(refresh_apps);
            }
            list
        }
        None => refresh_apps(),
    }
}

fn refresh_apps() -> Vec<AppEntry> {
    let list = std::thread::spawn(read_apps).join().unwrap_or_default();
    if !list.is_empty() {
        *APPS.lock().unwrap() = Some((Instant::now(), list.clone()));
    }
    list
}

/// Apps whose name matches the typed text, best matches first. Only the hits are cloned.
pub fn search_apps(query: &str, limit: usize) -> Vec<AppEntry> {
    let q = crate::search::fold(query);
    let guard = APPS.lock().unwrap();
    let Some((_, list)) = guard.as_ref() else { return Vec::new() };
    let mut hits: Vec<(u8, &AppEntry)> = list.iter().filter_map(|a| Some((crate::search::match_rank(&crate::search::fold(&a.name), &q)?, a))).collect();
    hits.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase())));
    hits.into_iter().take(limit).map(|(_, a)| a.clone()).collect()
}

/// Warms the app list at startup so the first Start opening is instant.
pub fn warm_up() {
    std::thread::spawn(|| {
        refresh_apps();
    });
}

// ---------------------------------------------------------------------------------------------
// Launching
// ---------------------------------------------------------------------------------------------

fn shell_open(target: &str) -> bool {
    let wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let result = unsafe { ShellExecuteW(None, windows::core::w!("open"), PCWSTR(wide.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
    // Values above 32 mean success.
    result.0 as isize > 32
}

fn display_name() -> String {
    use windows::Win32::Security::Authentication::Identity::{GetUserNameExW, NameDisplay};
    let mut buf = [0u16; 256];
    let mut len = buf.len() as u32;
    let ok = unsafe { GetUserNameExW(NameDisplay, Some(windows::core::PWSTR(buf.as_mut_ptr())), &mut len) };
    if ok && len > 0 {
        return String::from_utf16_lossy(&buf[..len as usize]);
    }
    std::env::var("USERNAME").unwrap_or_default()
}

// These two are `async` on purpose. Synchronous Tauri commands run on the main thread, and reading the app list
// asks shell components that live in the main thread's apartment: blocking the main thread while waiting for
// that read deadlocks the whole app.
#[tauri::command]
pub async fn launcher_apps() -> Vec<AppEntry> {
    tauri::async_runtime::spawn_blocking(apps).await.unwrap_or_default()
}

fn compute_state() -> StartState {
    let list = apps();
    let known = |id: &str| list.iter().any(|a| a.id == id);
    with_store(|store| {
        if store.pinned.is_empty() && store.usage.is_empty() {
            store.pinned = default_pins(&list);
        }
        StartState {
            pinned: store.pinned.iter().filter(|id| known(id)).cloned().collect(),
            recent: top_recent(&store.usage, &known, RECENT_COUNT),
            user: display_name(),
        }
    })
}

#[tauri::command]
pub async fn launcher_state() -> StartState {
    tauri::async_runtime::spawn_blocking(compute_state)
        .await
        .unwrap_or(StartState { pinned: Vec::new(), recent: Vec::new(), user: String::new() })
}

#[tauri::command]
pub fn launch_app(id: String) -> bool {
    let ok = shell_open(&format!("shell:AppsFolder\\{id}"));
    if ok {
        with_store(|store| {
            let usage = store.usage.entry(id).or_default();
            usage.count += 1;
            usage.last = now_secs();
        });
    }
    ok
}

#[tauri::command]
pub fn toggle_pin(id: String) {
    with_store(|store| toggle(&mut store.pinned, &id));
}

#[tauri::command]
pub fn open_location(kind: String) -> bool {
    location_target(&kind).is_some_and(shell_open)
}

#[tauri::command]
pub fn power_action(kind: String) -> bool {
    use std::os::windows::process::CommandExt;
    let Some((exe, args)) = power_command(&kind) else { return false };
    let system32 = PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into())).join("System32");
    std::process::Command::new(system32.join(exe)).args(args).creation_flags(0x0800_0000).spawn().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(pairs: &[(&str, u32, u64)]) -> HashMap<String, Usage> {
        pairs.iter().map(|(id, count, last)| (id.to_string(), Usage { count: *count, last: *last })).collect()
    }

    #[test]
    fn recent_apps_are_the_latest_used_and_skip_uninstalled_ones() {
        let u = usage(&[("a", 9, 10), ("b", 1, 30), ("c", 5, 20), ("gone", 99, 99)]);
        let known = |id: &str| id != "gone";
        assert_eq!(top_recent(&u, &known, 2), vec!["b", "c"]);
        assert_eq!(top_recent(&u, &known, 10), vec!["b", "c", "a"]);
    }

    #[test]
    fn equal_recency_is_ordered_by_use_count() {
        let u = usage(&[("a", 1, 5), ("b", 7, 5)]);
        assert_eq!(top_recent(&u, &|_| true, 2), vec!["b", "a"]);
    }

    #[test]
    fn toggling_pins_and_unpins() {
        let mut pins = vec!["a".to_string()];
        toggle(&mut pins, "b");
        assert_eq!(pins, vec!["a", "b"]);
        toggle(&mut pins, "a");
        assert_eq!(pins, vec!["b"]);
    }

    #[test]
    fn default_pins_match_known_names_in_either_language_without_duplicates() {
        let app = |id: &str, name: &str| AppEntry { id: id.into(), name: name.into(), icon_png_base64: None };
        let apps = vec![app("edge", "Microsoft Edge"), app("fe", "Dosya Gezgini"), app("x", "Some Game"), app("set", "Ayarlar")];
        assert_eq!(default_pins(&apps), vec!["edge", "fe", "set"]);
    }

    #[test]
    fn power_buttons_run_fixed_system_tools() {
        assert_eq!(power_command("shutdown"), Some(("shutdown.exe", &["/s", "/t", "0"][..])));
        assert_eq!(power_command("restart"), Some(("shutdown.exe", &["/r", "/t", "0"][..])));
        assert_eq!(power_command("signout"), Some(("shutdown.exe", &["/l"][..])));
        assert_eq!(power_command("lock").map(|c| c.0), Some("rundll32.exe"));
        assert_eq!(power_command("sleep").map(|c| c.0), Some("rundll32.exe"));
        assert_eq!(power_command("format c:"), None);
    }

    #[test]
    fn uninstall_shortcuts_are_hidden_but_apps_with_uninstall_in_their_name_stay() {
        assert!(is_uninstaller("Ağ Bağlantıları Yöneticisini Kaldır"));
        assert!(is_uninstaller("Uninstall Spotify"));
        assert!(is_uninstaller("Foo Uninstall"));
        assert!(!is_uninstaller("BCUninstaller"));
        assert!(!is_uninstaller("Bulk Crap Uninstaller"));
        assert!(!is_uninstaller("Ayarlar"));
    }

    #[test]
    fn only_known_shortcuts_open_something() {
        assert_eq!(location_target("settings"), Some("ms-settings:"));
        assert_eq!(location_target("downloads"), Some("shell:Downloads"));
        assert_eq!(location_target("cmd /c calc"), None);
    }
}
