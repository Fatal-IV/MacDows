//! The apps pinned to the Windows taskbar, shown in the dock. Windows keeps them in two places: shortcuts in
//! `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar` (desktop programs, File Explorer)
//! and the `Taskband\Favorites` registry value, which also names the pinned Store apps and holds the order.
//! MacDows reads both and follows changes; removing a pin goes through the Windows command first.
use serde::Serialize;
use std::path::PathBuf;
use tauri::menu::{ContextMenu, Menu, MenuItem};
use tauri::{AppHandle, Emitter, Manager};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::SIZE;
use windows::Win32::Storage::FileSystem::{GetShortPathNameW, WIN32_FIND_DATAW};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ};
use windows::Win32::UI::Shell::{IShellItem, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, ShellExecuteExW, ShellExecuteW, ShellLink, SHELLEXECUTEINFOW, SIGDN_NORMALDISPLAY, SIIGBF_ICONONLY};
use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNORMAL};

const APPS_FOLDER: &str = r"shell:AppsFolder\";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PinnedApp {
    /// A shortcut's full path, or `shell:AppsFolder\<AppUserModelID>` for a Store app; what the commands are given back.
    pub id: String,
    pub name: String,
    /// The program the pin starts ("" for a Store app); running windows are matched to it.
    pub exe: String,
    /// AppUserModelID of a Store app ("" otherwise); its windows are matched to it.
    pub aumid: String,
    pub icon_png_base64: Option<String>,
}

fn pinned_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_default();
    base.join(r"Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar")
}

// ---------------------------------------------------------------------------------------------
// The order blob
// ---------------------------------------------------------------------------------------------

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w == needle)
}

fn find_ignore_case(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || hay.len() < needle.len() {
        return None;
    }
    hay.windows(needle.len()).position(|w| w.eq_ignore_ascii_case(needle))
}

fn utf16le(text: &str) -> Vec<u8> {
    text.encode_utf16().flat_map(|c| c.to_le_bytes()).collect()
}

/// Where the `Favorites` blob first mentions a shortcut: by its long name (UTF-16 or plain) or by its
/// 8.3 short name (File Explorer is only listed as `FILEEX~1.LNK`). Earlier means further left in the taskbar.
pub fn pin_position(blob: &[u8], long: &str, short: &str) -> Option<usize> {
    [find(blob, &utf16le(long)), find(blob, long.as_bytes()), find_ignore_case(blob, short.as_bytes()), find_ignore_case(blob, &utf16le(short))]
        .into_iter()
        .flatten()
        .min()
}

fn is_aumid(text: &str) -> bool {
    let Some((family, app)) = text.split_once('!') else { return false };
    if app.is_empty() || app.contains('!') {
        return false;
    }
    let Some((name, publisher)) = family.rsplit_once('_') else { return false };
    !name.is_empty() && publisher.len() == 13 && publisher.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// The Store apps named in the blob (`Package_publisherid!App`), with the byte position of the first mention.
/// The strings are UTF-16 and may sit on an odd byte, so both alignments are read.
pub fn scan_aumids(blob: &[u8]) -> Vec<(usize, String)> {
    let mut found: Vec<(usize, String)> = Vec::new();
    for align in 0..2usize {
        let units: Vec<u16> = blob.get(align..).unwrap_or_default().chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        let allowed = |u: u16| u < 128 && ((u as u8).is_ascii_alphanumeric() || matches!(u as u8, b'.' | b'_' | b'-' | b'!'));
        let mut start = 0;
        while start < units.len() {
            if !allowed(units[start]) {
                start += 1;
                continue;
            }
            let mut end = start;
            while end < units.len() && allowed(units[end]) {
                end += 1;
            }
            let run: String = units[start..end].iter().map(|&u| u as u8 as char).collect();
            if is_aumid(&run) {
                found.push((align + 2 * start, run));
            }
            start = end;
        }
    }
    // The same app can be named more than once: keep its first mention.
    found.sort_by_key(|(pos, _)| *pos);
    let mut seen: Vec<String> = Vec::new();
    found.retain(|(_, id)| {
        let key = id.to_lowercase();
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
    found
}

/// Items in taskbar order; those the blob does not mention go last, alphabetically.
pub fn sort_pins<T>(mut items: Vec<(Option<usize>, String, T)>) -> Vec<T> {
    items.sort_by(|a, b| match (a.0, b.0) {
        (Some(x), Some(y)) => x.cmp(&y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.1.to_lowercase().cmp(&b.1.to_lowercase()),
    });
    items.into_iter().map(|(_, _, t)| t).collect()
}

fn is_shortcut(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".lnk")
}

fn favorites_blob() -> Vec<u8> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband")
        .and_then(|k| k.get_raw_value("Favorites"))
        .map(|v| v.bytes)
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// Reading the pins
// ---------------------------------------------------------------------------------------------

enum Pin {
    Shortcut(String),
    Store(String),
}

fn short_name(path: &std::path::Path) -> String {
    let wide: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    let mut buf = [0u16; 520];
    let n = unsafe { GetShortPathNameW(PCWSTR(wide.as_ptr()), Some(&mut buf)) } as usize;
    if n == 0 || n > buf.len() {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..n]).rsplit('\\').next().unwrap_or_default().to_string()
}

/// Everything pinned right now, in taskbar order (nothing is resolved yet).
fn pins() -> Vec<Pin> {
    let blob = favorites_blob();
    let dir = pinned_dir();
    let mut items: Vec<(Option<usize>, String, Pin)> = Vec::new();
    if let Ok(read) = std::fs::read_dir(&dir) {
        for file in read.flatten().filter_map(|e| e.file_name().into_string().ok()).filter(|n| is_shortcut(n)) {
            let short = short_name(&dir.join(&file));
            items.push((pin_position(&blob, &file, &short), file.clone(), Pin::Shortcut(file)));
        }
    }
    for (pos, aumid) in scan_aumids(&blob) {
        items.push((Some(pos), aumid.clone(), Pin::Store(aumid)));
    }
    sort_pins(items)
}

/// The program a `.lnk` file starts. File Explorer's shortcut has no file target (it points at a shell folder): it is `explorer.exe`.
fn shortcut_target(path: &std::path::Path) -> Option<String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).ok()?;
        let file: IPersistFile = windows::core::Interface::cast(&link).ok()?;
        let wide: Vec<u16> = path.as_os_str().to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
        file.Load(PCWSTR(wide.as_ptr()), STGM_READ).ok()?;
        let mut buf = [0u16; 520];
        let mut data = WIN32_FIND_DATAW::default();
        let _ = link.GetPath(&mut buf, &mut data, 0);
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        let target = String::from_utf16_lossy(&buf[..len]);
        if !target.is_empty() {
            return Some(target);
        }
    }
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    Some(format!(r"{root}\explorer.exe"))
}

/// Name and icon of a Store app from the shell's Apps folder; `None` when the app is not installed any more.
fn store_info(aumid: &str) -> Option<(String, Option<String>)> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let wide: Vec<u16> = format!("{APPS_FOLDER}{aumid}").encode_utf16().chain(std::iter::once(0)).collect();
        let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None).ok()?;
        let name_ptr = item.GetDisplayName(SIGDN_NORMALDISPLAY).ok()?;
        let name = name_ptr.to_string().ok();
        CoTaskMemFree(Some(name_ptr.0 as *const _));
        let icon = windows::core::Interface::cast::<IShellItemImageFactory>(&item).ok().and_then(|factory| {
            let bitmap = factory.GetImage(SIZE { cx: 64, cy: 64 }, SIIGBF_ICONONLY).ok()?;
            let png = crate::windows_list::hbitmap_to_png_base64(bitmap);
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(bitmap.into());
            png
        });
        Some((name?, icon))
    }
}

fn read_pinned() -> Vec<PinnedApp> {
    let dir = pinned_dir();
    pins()
        .into_iter()
        .filter_map(|pin| match pin {
            Pin::Shortcut(file) => {
                let path = dir.join(&file);
                let exe = shortcut_target(&path)?;
                let icon = crate::windows_list::icon_png_base64(&exe);
                Some(PinnedApp { id: path.to_string_lossy().into_owned(), name: file.trim_end_matches(".lnk").trim_end_matches(".LNK").to_string(), exe, aumid: String::new(), icon_png_base64: icon })
            }
            Pin::Store(aumid) => {
                let (name, icon) = store_info(&aumid)?;
                Some(PinnedApp { id: format!("{APPS_FOLDER}{aumid}"), name, exe: String::new(), aumid, icon_png_base64: icon })
            }
        })
        .collect()
}

/// Pins the user hid in the dock (see `unpin`) are left out.
#[tauri::command]
pub async fn list_pinned(app: AppHandle) -> Vec<PinnedApp> {
    let hidden: Vec<String> = app.state::<crate::AppSettings>().0.lock().unwrap().hidden_pinned.iter().map(|h| h.to_lowercase()).collect();
    let all = tauri::async_runtime::spawn_blocking(read_pinned).await.unwrap_or_default();
    all.into_iter().filter(|p| !hidden.contains(&p.id.to_lowercase())).collect()
}

/// Is this id one of the pins Windows has right now? Commands only act on listed pins.
fn is_listed(id: &str) -> bool {
    if let Some(aumid) = id.strip_prefix(APPS_FOLDER) {
        return scan_aumids(&favorites_blob()).iter().any(|(_, a)| a.eq_ignore_ascii_case(aumid));
    }
    let path = PathBuf::from(id);
    path.parent() == Some(pinned_dir().as_path()) && path.file_name().and_then(|n| n.to_str()).is_some_and(is_shortcut) && path.exists()
}

/// Starts a pinned app.
#[tauri::command]
pub fn launch_pinned(id: String) -> Result<(), String> {
    if !is_listed(&id) {
        return Err("not a pinned app".into());
    }
    let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        ShellExecuteW(None, w!("open"), PCWSTR(wide.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Following changes
// ---------------------------------------------------------------------------------------------

/// What changes when something is pinned, unpinned or reordered: shortcut names and write times, and the blob.
fn fingerprint() -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    if let Ok(read) = std::fs::read_dir(pinned_dir()) {
        let mut files: Vec<(String, Option<std::time::SystemTime>)> = read.flatten().map(|e| (e.file_name().to_string_lossy().into_owned(), e.metadata().and_then(|m| m.modified()).ok())).collect();
        files.sort();
        files.hash(&mut h);
    }
    favorites_blob().hash(&mut h);
    h.finish()
}

/// Tells the dock to reload when the pinned apps change in Windows (checked every 3 seconds).
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = fingerprint();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let now = fingerprint();
            if now != last {
                last = now;
                let _ = app.emit("pinned-changed", ());
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------
// Removing a pin (right click in the dock)
// ---------------------------------------------------------------------------------------------

const UNPIN_PREFIX: &str = "pinned-unpin:";

/// Asks Windows to unpin through its own "Unpin from taskbar" command. Returns whether the call was accepted.
fn windows_unpin(id: &str) -> bool {
    let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
    let mut info = SHELLEXECUTEINFOW { cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32, lpVerb: w!("taskbarunpin"), lpFile: PCWSTR(wide.as_ptr()), nShow: SW_HIDE.0, ..Default::default() };
    unsafe { ShellExecuteExW(&mut info).is_ok() }
}

/// Removes a pin: first Windows's own command; when Windows does not honor it (Windows 11 often refuses
/// other programs), the pin is hidden in the dock only and the Windows taskbar list is left alone.
fn unpin(app: &AppHandle, id: String) {
    if !is_listed(&id) {
        return;
    }
    let accepted = windows_unpin(&id);
    // Explorer updates its list a moment after the command.
    for _ in 0..6 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if !is_listed(&id) {
            eprintln!("[pinned] {id}: removed through Windows");
            let _ = app.emit("pinned-changed", ());
            return;
        }
    }
    eprintln!("[pinned] {id}: Windows kept it (command accepted: {accepted}); hidden in the dock only");
    let state = app.state::<crate::AppSettings>();
    let mut guard = state.0.lock().unwrap();
    if !guard.hidden_pinned.iter().any(|h| h.eq_ignore_ascii_case(&id)) {
        guard.hidden_pinned.push(id);
    }
    let _ = crate::settings::save(&guard);
    let updated = guard.clone();
    drop(guard);
    let _ = app.emit("settings-changed", updated);
    let _ = app.emit("pinned-changed", ());
}

/// Shows the right-click menu of a pinned dock icon.
#[tauri::command]
pub fn pinned_menu(app: AppHandle, id: String) -> Result<(), String> {
    let lang = app.state::<crate::AppSettings>().0.lock().unwrap().language.clone();
    let item = MenuItem::with_id(&app, format!("{UNPIN_PREFIX}{id}"), crate::i18n::unpin_label(&lang), true, None::<&str>).map_err(|e| e.to_string())?;
    let menu = Menu::with_items(&app, &[&item]).map_err(|e| e.to_string())?;
    let window = app.get_webview_window("dock").ok_or("no dock window")?;
    menu.popup(window.as_ref().window().clone()).map_err(|e| e.to_string())
}

/// App-wide menu events: the dock's "Unpin from taskbar".
pub fn on_menu_event(app: &AppHandle, menu_id: &str) {
    if let Some(id) = menu_id.strip_prefix(UNPIN_PREFIX) {
        let (app, id) = (app.clone(), id.to_string());
        std::thread::spawn(move || unpin(&app, id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The strings the real blob of this PC holds, at the byte positions where they were found.
    fn blob() -> Vec<u8> {
        let mut b = vec![0u8; 4000];
        let mut put = |at: usize, bytes: Vec<u8>| b[at..at + bytes.len()].copy_from_slice(&bytes);
        put(10, b"FILEEX~1.LNK".to_vec());
        put(223, utf16le("File Explorer.lnk"));
        put(648, utf16le("Microsoft Edge.lnk"));
        put(1120, utf16le("Microsoft.WindowsStore_8wekyb3d8bbwe!App"));
        put(2761, utf16le("AD2F1837.HPSupportAssistant_v10z8vjag6ke6!AD2F1837.HPSupportAssistant"));
        b
    }

    #[test]
    fn explorer_is_found_by_its_short_name_too() {
        let b = blob();
        assert_eq!(pin_position(&b, "File Explorer.lnk", "FILEEX~1.LNK"), Some(10));
        // Without the short name in the blob the long name still counts.
        assert_eq!(pin_position(&b, "Microsoft Edge.lnk", "MICROS~1.LNK"), Some(648));
        assert_eq!(pin_position(&b, "Nope.lnk", "NOPE~1.LNK"), None);
    }

    #[test]
    fn store_apps_are_read_from_either_byte_alignment() {
        let found = scan_aumids(&blob());
        let ids: Vec<&str> = found.iter().map(|(_, id)| id.as_str()).collect();
        assert_eq!(ids, ["Microsoft.WindowsStore_8wekyb3d8bbwe!App", "AD2F1837.HPSupportAssistant_v10z8vjag6ke6!AD2F1837.HPSupportAssistant"]);
        assert!(found[0].0 < found[1].0);
    }

    #[test]
    fn a_store_app_named_twice_counts_once_at_its_first_mention() {
        let mut b = vec![0u8; 400];
        let id = utf16le("Pkg.App_8wekyb3d8bbwe!App");
        b[20..20 + id.len()].copy_from_slice(&id);
        b[200..200 + id.len()].copy_from_slice(&id);
        let found = scan_aumids(&b);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, 20);
    }

    #[test]
    fn only_real_app_ids_are_taken() {
        assert!(is_aumid("Microsoft.WindowsStore_8wekyb3d8bbwe!App"));
        assert!(!is_aumid("Microsoft.WindowsStore_8wekyb3d8bbwe"));
        assert!(!is_aumid("Assets-AppTiles!x"));
        assert!(!is_aumid("Name_short!App"));
        assert!(!is_aumid("Name_8wekyb3d8bbwe!"));
    }

    #[test]
    fn pins_sort_by_position_then_unknown_alphabetically() {
        let items = vec![(Some(900), "b".to_string(), 'B'), (None, "zed".to_string(), 'Z'), (Some(5), "a".to_string(), 'A'), (None, "Alpha".to_string(), 'L')];
        assert_eq!(sort_pins(items), vec!['A', 'B', 'L', 'Z']);
    }

    #[test]
    fn only_lnk_files_are_shortcuts() {
        assert!(is_shortcut("Edge.LNK"));
        assert!(!is_shortcut("desktop.ini"));
    }

    #[test]
    fn only_listed_pins_are_acted_on() {
        assert!(!is_listed(r"C:\Windows\System32\cmd.exe"));
        assert!(!is_listed(r"shell:AppsFolder\Not.Pinned_8wekyb3d8bbwe!App"));
    }
}
