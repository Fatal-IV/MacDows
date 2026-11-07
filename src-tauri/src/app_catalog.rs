//! "Add app": the list of installed apps the dock can pin, and the panel that shows it. Desktop programs come
//! from the Start menu shortcuts (which name the program, so running windows can be matched), Store apps from the
//! shell's Apps folder (which names their app id).
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

use crate::pinned_apps::{same_app, PinnedApp};

const CACHE_FOR: Duration = Duration::from_secs(30);
const PANEL_W: f64 = 360.0;
const PANEL_H: f64 = 440.0;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CatalogApp {
    /// `exe:<path lowercase>` or `aumid:<id lowercase>`; what `pin_catalog_app` is given back.
    pub id: String,
    pub name: String,
    /// The program ("" for a Store app).
    pub exe: String,
    /// The Store app id ("" for a desktop program).
    pub aumid: String,
    pub icon_png_base64: Option<String>,
    /// Has a window open right now.
    pub running: bool,
    /// Already pinned (to the Windows taskbar or by MacDows).
    pub pinned: bool,
}

// ---------------------------------------------------------------------------------------------
// Pure parts
// ---------------------------------------------------------------------------------------------

/// Start menu entries that are not apps: uninstallers, help files, web links.
pub fn is_noise(name: &str) -> bool {
    let n = name.to_lowercase();
    ["uninstall", "kaldır", "readme", "release notes", "documentation", "help", "yardım", "website", "web sitesi", "manual", "license"].iter().any(|bad| n.contains(bad))
}

/// Start menu shortcuts to a folder or a settings page start `explorer.exe`; they are not programs of their own
/// (and would be mistaken for File Explorer itself).
pub fn is_shell_host(exe: &str) -> bool {
    exe.replace('/', "\\").rsplit('\\').next().is_some_and(|file| file.eq_ignore_ascii_case("explorer.exe"))
}

/// Apps whose name contains the query (case-insensitive; an empty query matches all): running ones first, then
/// alphabetical.
pub fn filter_apps(apps: &[CatalogApp], query: &str) -> Vec<CatalogApp> {
    let q = query.trim().to_lowercase();
    let mut out: Vec<CatalogApp> = apps.iter().filter(|a| q.is_empty() || a.name.to_lowercase().contains(&q)).cloned().collect();
    out.sort_by(|a, b| b.running.cmp(&a.running).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase())));
    out
}

/// One entry per program or Store app: the first one seen stays.
pub fn dedupe(apps: Vec<CatalogApp>) -> Vec<CatalogApp> {
    let mut seen: Vec<String> = Vec::new();
    apps.into_iter()
        .filter(|a| {
            let key = if a.aumid.is_empty() { a.exe.replace('/', "\\").to_lowercase() } else { a.aumid.to_lowercase() };
            if seen.contains(&key) {
                false
            } else {
                seen.push(key);
                true
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Reading the installed apps
// ---------------------------------------------------------------------------------------------

fn start_menu_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(p) = std::env::var_os("ProgramData") {
        dirs.push(PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    if let Some(p) = std::env::var_os("APPDATA") {
        dirs.push(PathBuf::from(p).join(r"Microsoft\Windows\Start Menu\Programs"));
    }
    dirs
}

fn collect_shortcuts(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 4 {
        return;
    }
    let Ok(read) = std::fs::read_dir(dir) else { return };
    for entry in read.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_shortcuts(&path, out, depth + 1);
        } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("lnk")) {
            out.push(path);
        }
    }
}

/// Store apps (`Package_publisher!App`) the shell lists, with their display names.
fn store_apps() -> Vec<(String, String)> {
    use windows::core::{w, GUID};
    use windows::Win32::System::Com::{CoInitializeEx, CoTaskMemFree, COINIT_APARTMENTTHREADED};
    use windows::Win32::UI::Shell::{IEnumShellItems, IShellItem, SHCreateItemFromParsingName, SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVEPARSING};
    const BHID_ENUM_ITEMS: GUID = GUID::from_u128(0x94f60519_2850_4924_aa5a_d15e84868039);
    let mut out = Vec::new();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let Ok(folder) = SHCreateItemFromParsingName::<_, _, IShellItem>(w!("shell:AppsFolder"), None) else { return out };
        let Ok(items) = folder.BindToHandler::<_, IEnumShellItems>(None, &BHID_ENUM_ITEMS) else { return out };
        loop {
            let mut batch: [Option<IShellItem>; 16] = Default::default();
            let mut got = 0u32;
            let _ = items.Next(&mut batch, Some(&mut got));
            if got == 0 {
                break;
            }
            for item in batch.iter().take(got as usize).flatten() {
                let (Ok(id_ptr), Ok(name_ptr)) = (item.GetDisplayName(SIGDN_PARENTRELATIVEPARSING), item.GetDisplayName(SIGDN_NORMALDISPLAY)) else { continue };
                let (id, name) = (id_ptr.to_string().unwrap_or_default(), name_ptr.to_string().unwrap_or_default());
                CoTaskMemFree(Some(id_ptr.0 as *const _));
                CoTaskMemFree(Some(name_ptr.0 as *const _));
                if id.contains('!') && !name.is_empty() {
                    out.push((id, name));
                }
            }
        }
    }
    out
}

/// Every app that could be pinned, without pinned/running flags (see `flag`).
fn read_installed() -> Vec<CatalogApp> {
    let mut apps: Vec<CatalogApp> = Vec::new();
    let mut shortcuts = Vec::new();
    for dir in start_menu_dirs() {
        collect_shortcuts(&dir, &mut shortcuts, 0);
    }
    for lnk in shortcuts {
        let name = lnk.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        if name.is_empty() || is_noise(&name) {
            continue;
        }
        let Some(exe) = crate::pinned_apps::lnk_target(&lnk) else { continue };
        if !exe.to_lowercase().ends_with(".exe") || is_shell_host(&exe) || !Path::new(&exe).exists() {
            continue;
        }
        let icon = crate::windows_list::icon_png_base64(&exe);
        apps.push(CatalogApp { id: format!("exe:{}", exe.replace('/', "\\").to_lowercase()), name, exe, aumid: String::new(), icon_png_base64: icon, running: false, pinned: false });
    }
    for (aumid, name) in store_apps() {
        if is_noise(&name) {
            continue;
        }
        let icon = crate::pinned_apps::store_info(&aumid).and_then(|(_, icon)| icon);
        apps.push(CatalogApp { id: format!("aumid:{}", aumid.to_lowercase()), name, exe: String::new(), aumid, icon_png_base64: icon, running: false, pinned: false });
    }
    dedupe(apps)
}

static CACHE: Mutex<Option<(Instant, Vec<CatalogApp>)>> = Mutex::new(None);

fn installed_cached() -> Vec<CatalogApp> {
    let mut guard = CACHE.lock().unwrap();
    if let Some((at, apps)) = guard.as_ref() {
        if at.elapsed() < CACHE_FOR {
            return apps.clone();
        }
    }
    let apps = read_installed();
    *guard = Some((Instant::now(), apps.clone()));
    apps
}

/// Marks which apps are pinned and which have a window open.
fn flag(mut apps: Vec<CatalogApp>, pins: &[PinnedApp], windows: &[crate::windows_list::AppWindow]) -> Vec<CatalogApp> {
    for a in &mut apps {
        a.pinned = pins.iter().any(|p| same_app(p, &a.exe, &a.aumid));
        a.running = windows.iter().any(|w| (!a.exe.is_empty() && !w.exe.is_empty() && w.exe.replace('/', "\\").eq_ignore_ascii_case(&a.exe.replace('/', "\\"))) || (!a.aumid.is_empty() && w.aumid.eq_ignore_ascii_case(&a.aumid)));
    }
    apps
}

/// The installed apps with their pinned and running flags, running ones first.
#[tauri::command]
pub async fn get_catalog(app: AppHandle) -> Vec<CatalogApp> {
    let local = app.state::<crate::AppSettings>().0.lock().unwrap().dock.local_pins.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let pins = crate::pinned_apps::merge_local(crate::pinned_apps::read_pinned(), &local);
        let windows = crate::windows_list::enumerate();
        filter_apps(&flag(installed_cached(), &pins, &windows), "")
    })
    .await
    .unwrap_or_default()
}

/// Pins the app (to the Windows taskbar when Windows lets it, otherwise in MacDows), then the dock reloads.
#[tauri::command]
pub fn pin_catalog_app(app: AppHandle, id: String) {
    std::thread::spawn(move || {
        let found = installed_cached().into_iter().find(|a| a.id.eq_ignore_ascii_case(&id));
        if let Some(a) = found {
            crate::pinned_apps::pin_app(&app, &a.exe, &a.aumid, &a.name);
        }
    });
}

// ---------------------------------------------------------------------------------------------
// The panel
// ---------------------------------------------------------------------------------------------

/// Opens (or closes, if open) the panel above the middle of the dock `dock_label` (one dock per monitor).
pub fn toggle_panel(app: &AppHandle, dock_label: &str) {
    if let Some(existing) = app.get_webview_window("addapp") {
        let _ = existing.destroy();
        return;
    }
    let Some(dock) = app.get_webview_window(dock_label) else { return };
    let (Ok(pos), Ok(size), Ok(Some(monitor))) = (dock.outer_position(), dock.outer_size(), dock.current_monitor()) else { return };
    let scale = dock.scale_factor().unwrap_or(1.0);
    let work = monitor.work_area();
    let x = (pos.x as f64 + size.width as f64 / 2.0 - PANEL_W * scale / 2.0).clamp(work.position.x as f64 + 8.0, (work.position.x + work.size.width as i32) as f64 - PANEL_W * scale - 8.0);
    let y = (pos.y as f64 - PANEL_H * scale - 10.0 * scale).max(work.position.y as f64 + 8.0);
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let built = WebviewWindowBuilder::new(&handle, "addapp", WebviewUrl::App("index.html".into()))
            .title("MacDows apps")
            .inner_size(PANEL_W, PANEL_H)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(true)
            .visible(false)
            .build();
        if let Ok(win) = built {
            if let Ok(hwnd) = win.hwnd() {
                crate::windows_list::register_bar(hwnd.0 as isize);
            }
            let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
            let _ = win.show();
            let _ = win.set_focus();
        }
    });
}

#[tauri::command]
pub fn close_add_app_panel(app: AppHandle) {
    if let Some(w) = app.get_webview_window("addapp") {
        let _ = w.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(name: &str, exe: &str, aumid: &str, running: bool) -> CatalogApp {
        CatalogApp { id: format!("x:{name}"), name: name.into(), exe: exe.into(), aumid: aumid.into(), icon_png_base64: None, running, pinned: false }
    }

    #[test]
    fn running_apps_come_first_then_alphabetical_ignoring_case() {
        let apps = vec![app("zebra", "z.exe", "", false), app("Alpha", "a.exe", "", false), app("Mid", "m.exe", "", true)];
        let names: Vec<String> = filter_apps(&apps, "").into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["Mid", "Alpha", "zebra"]);
    }

    #[test]
    fn the_query_matches_part_of_a_name_in_any_case() {
        let apps = vec![app("Calculator", "c.exe", "", false), app("Paint", "p.exe", "", false)];
        assert_eq!(filter_apps(&apps, "  CALC ").len(), 1);
        assert_eq!(filter_apps(&apps, "alc")[0].name, "Calculator");
        assert!(filter_apps(&apps, "nothing").is_empty());
    }

    #[test]
    fn duplicates_by_program_or_app_id_keep_the_first() {
        let apps = vec![app("A", r"C:\x\a.exe", "", false), app("A again", "c:/x/A.EXE", "", false), app("S", "", "Pkg_8wekyb3d8bbwe!App", false), app("S2", "", "pkg_8wekyb3d8bbwe!app", false)];
        let names: Vec<String> = dedupe(apps).into_iter().map(|a| a.name).collect();
        assert_eq!(names, ["A", "S"]);
    }

    #[test]
    fn uninstallers_and_help_files_are_not_apps() {
        for bad in ["Uninstall Foo", "Foo kaldır", "Foo Readme", "Foo Help", "Foo Release Notes", "Foo Website"] {
            assert!(is_noise(bad), "{bad}");
        }
        for good in ["Calculator", "Visual Studio Code", "Paint"] {
            assert!(!is_noise(good), "{good}");
        }
    }

    #[test]
    fn folder_shortcuts_that_start_explorer_are_not_apps() {
        assert!(is_shell_host(r"C:\WINDOWS\explorer.exe"));
        assert!(is_shell_host("c:/windows/Explorer.EXE"));
        assert!(!is_shell_host(r"C:\Program Files\Foo\explorer_helper.exe"));
        assert!(!is_shell_host(r"C:\Program Files\Foo\foo.exe"));
    }

    #[test]
    fn flags_mark_pinned_and_running_apps() {
        let pins = vec![PinnedApp { id: "p".into(), name: "A".into(), exe: r"C:\x\a.exe".into(), aumid: String::new(), icon_png_base64: None }];
        let windows = vec![crate::windows_list::AppWindow { id: 1, title: "t".into(), exe: "c:/x/B.exe".into(), aumid: String::new(), icon_png_base64: None }];
        let flagged = flag(vec![app("A", r"C:\x\a.exe", "", false), app("B", r"C:\x\b.exe", "", false), app("C", r"C:\x\c.exe", "", false)], &pins, &windows);
        assert_eq!((flagged[0].pinned, flagged[0].running), (true, false));
        assert_eq!((flagged[1].pinned, flagged[1].running), (false, true));
        assert_eq!((flagged[2].pinned, flagged[2].running), (false, false));
    }

    /// Run by hand: `cargo test prints_the_installed_apps -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn prints_the_installed_apps() {
        let started = std::time::Instant::now();
        let apps = read_installed();
        let desktop = apps.iter().filter(|a| a.aumid.is_empty()).count();
        println!("{} apps in {:?}: {} desktop programs, {} Store apps", apps.len(), started.elapsed(), desktop, apps.len() - desktop);
        for a in apps.iter().take(12) {
            println!("  {} | exe={} | aumid={} | icon={}", a.name, a.exe, a.aumid, a.icon_png_base64.is_some());
        }
    }
}
