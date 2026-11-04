//! The apps pinned to the Windows taskbar, shown in the dock. Windows keeps them in two places: shortcuts in
//! `%APPDATA%\Microsoft\Internet Explorer\Quick Launch\User Pinned\TaskBar` (desktop programs, File Explorer)
//! and the `Taskband\Favorites` registry value, which also names the pinned Store apps and holds the order.
//! MacDows reads both and follows changes; removing a pin goes through the Windows command first.
use serde::Serialize;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, SIZE, WPARAM};
use windows::Win32::Storage::FileSystem::{GetShortPathNameW, WIN32_FIND_DATAW};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, STGM_READ};
use windows::Win32::UI::Shell::{IShellItem, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, ShellExecuteExW, ShellExecuteW, ShellLink, SHELLEXECUTEINFOW, SIGDN_NORMALDISPLAY, SIIGBF_ICONONLY};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, SW_HIDE, SW_SHOWNORMAL, WM_CLOSE};

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

/// The file a `.lnk` shortcut starts; `None` when it has no file target (it points at a shell folder or a Store app).
pub fn lnk_target(path: &std::path::Path) -> Option<String> {
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
        (!target.is_empty()).then_some(target)
    }
}

/// The program a pinned `.lnk` starts. File Explorer's shortcut has no file target (it points at a shell folder): it is `explorer.exe`.
fn shortcut_target(path: &std::path::Path) -> Option<String> {
    if let Some(target) = lnk_target(path) {
        return Some(target);
    }
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    Some(format!(r"{root}\explorer.exe"))
}

/// Name and icon of a Store app from the shell's Apps folder; `None` when the app is not installed any more.
pub fn store_info(aumid: &str) -> Option<(String, Option<String>)> {
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

pub fn read_pinned() -> Vec<PinnedApp> {
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

/// Paths compare ignoring case and slash style.
fn norm(path: &str) -> String {
    path.replace('/', "\\").to_lowercase()
}

/// Is this pin the program `exe` or the Store app `aumid`? Empty values never match.
pub fn same_app(pin: &PinnedApp, exe: &str, aumid: &str) -> bool {
    (!exe.is_empty() && !pin.exe.is_empty() && norm(&pin.exe) == norm(exe)) || (!aumid.is_empty() && !pin.aumid.is_empty() && pin.aumid.eq_ignore_ascii_case(aumid))
}

/// The pins only MacDows has (Windows refused to pin them) follow the Windows pins; one Windows has since pinned is not listed twice.
/// Local pins come without an icon; `list_pinned` fills it in.
pub fn merge_local(mut pins: Vec<PinnedApp>, local: &[crate::settings::LocalPin]) -> Vec<PinnedApp> {
    for l in local {
        if pins.iter().any(|p| same_app(p, &l.exe, &l.aumid)) {
            continue;
        }
        pins.push(PinnedApp { id: l.id.clone(), name: l.name.clone(), exe: l.exe.clone(), aumid: l.aumid.clone(), icon_png_base64: None });
    }
    pins
}

/// The user's own order (ids saved by dragging; compared ignoring case): listed pins first in that order, the
/// others after them in their current order, saved ids that are no longer pinned ignored.
pub fn apply_order(pins: Vec<PinnedApp>, saved: &[String]) -> Vec<PinnedApp> {
    let rank = |id: &str| saved.iter().position(|s| s.eq_ignore_ascii_case(id)).unwrap_or(usize::MAX);
    let mut pins = pins;
    pins.sort_by_key(|p| rank(&p.id)); // stable: unlisted pins keep their relative order at the end
    pins
}

/// Saves the order the user dragged the pinned apps into (their ids, left to right). The dock reloads.
#[tauri::command]
pub fn set_pinned_order(app: AppHandle, ids: Vec<String>) {
    update_settings(&app, |s| s.dock.pinned_order = ids);
}

fn fill_local_icons(pins: &mut [PinnedApp]) {
    for p in pins.iter_mut().filter(|p| p.id.starts_with(LOCAL_PREFIX) && p.icon_png_base64.is_none()) {
        p.icon_png_base64 = if p.aumid.is_empty() { crate::windows_list::icon_png_base64(&p.exe) } else { store_info(&p.aumid).and_then(|(_, icon)| icon) };
    }
}

const LOCAL_PREFIX: &str = "local:";

/// Pins the user hid in the dock (see `unpin`) are left out; pins only MacDows has come after the Windows ones.
#[tauri::command]
pub async fn list_pinned(app: AppHandle) -> Vec<PinnedApp> {
    let (hidden, local, order): (Vec<String>, Vec<crate::settings::LocalPin>, Vec<String>) = {
        let state = app.state::<crate::AppSettings>();
        let g = state.0.lock().unwrap();
        (g.hidden_pinned.iter().map(|h| h.to_lowercase()).collect(), g.dock.local_pins.clone(), g.dock.pinned_order.clone())
    };
    let all = tauri::async_runtime::spawn_blocking(move || {
        let mut merged = merge_local(read_pinned(), &local);
        fill_local_icons(&mut merged);
        apply_order(merged, &order)
    })
    .await
    .unwrap_or_default();
    all.into_iter().filter(|p| !hidden.contains(&p.id.to_lowercase())).collect()
}

/// What to hand to the shell to start (a new instance of) an app: a Windows pin's own id, or for a program only MacDows
/// knows (local pin, running window) its Store app id or exe.
pub fn launch_target(id: &str, exe: &str, aumid: &str) -> Option<String> {
    if !id.is_empty() && !id.starts_with(LOCAL_PREFIX) {
        return Some(id.to_string());
    }
    if !aumid.is_empty() {
        return Some(format!("{APPS_FOLDER}{aumid}"));
    }
    (!exe.is_empty()).then(|| exe.to_string())
}

/// What the "pin to taskbar" command is given: a Store app is pinned by its app id (its windows are hosted by
/// `ApplicationFrameHost.exe`, which must never be pinned), anything else by its exe.
pub fn pin_source(exe: &str, aumid: &str) -> Option<String> {
    let host = exe.to_lowercase().ends_with("applicationframehost.exe");
    if !exe.is_empty() && !host {
        return Some(exe.to_string());
    }
    (!aumid.is_empty()).then(|| format!("{APPS_FOLDER}{aumid}"))
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
pub fn launch_pinned(app: AppHandle, id: String) -> Result<(), String> {
    if let Some(local) = id.starts_with(LOCAL_PREFIX).then(|| app.state::<crate::AppSettings>().0.lock().unwrap().dock.local_pins.iter().find(|l| l.id.eq_ignore_ascii_case(&id)).cloned()).flatten() {
        return shell_open(&launch_target(&id, &local.exe, &local.aumid).ok_or("nothing to start")?);
    }
    if !is_listed(&id) {
        return Err("not a pinned app".into());
    }
    shell_open(&id)
}

fn shell_open(target: &str) -> Result<(), String> {
    let wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        ShellExecuteW(None, w!("open"), PCWSTR(wide.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
    }
    Ok(())
}

/// The dock asks for a new window of a pinned app (Win+Shift+number).
#[tauri::command]
pub fn launch_new_app(id: String, exe: String, aumid: String) {
    launch_new(&id, &exe, &aumid);
}

/// Starts a new instance (a new window for most apps) of a pinned or running app.
pub fn launch_new(id: &str, exe: &str, aumid: &str) {
    let target = if id.starts_with(LOCAL_PREFIX) {
        launch_target(id, exe, aumid)
    } else if !id.is_empty() && !is_listed(id) {
        None
    } else {
        launch_target(id, exe, aumid)
    };
    if let Some(target) = target {
        let _ = shell_open(&target);
    }
}

/// Asks every window to close (the app may ask to save first); nothing is killed.
pub fn close_windows(hwnds: &[isize]) {
    for &h in hwnds {
        unsafe {
            let _ = PostMessageW(Some(HWND(h as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
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

/// Asks the shell to pin or unpin through its own taskbar command (`taskbarpin`, `taskbarunpin`). Returns whether the call was accepted.
fn windows_verb(verb: PCWSTR, target: &str) -> bool {
    let wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let mut info = SHELLEXECUTEINFOW { cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32, lpVerb: verb, lpFile: PCWSTR(wide.as_ptr()), nShow: SW_HIDE.0, ..Default::default() };
    unsafe { ShellExecuteExW(&mut info).is_ok() }
}

/// Removes a pin: Windows's own command first; when Windows does not honor it (Windows 11 often refuses
/// other programs), the pin is hidden in the dock only and the Windows taskbar list is left alone.
/// A pin only MacDows has is simply deleted from its own list.
pub fn remove_pin(app: &AppHandle, id: String) {
    if id.starts_with(LOCAL_PREFIX) {
        update_settings(app, |s| s.dock.local_pins.retain(|l| !l.id.eq_ignore_ascii_case(&id)));
        return;
    }
    if !is_listed(&id) {
        return;
    }
    let accepted = windows_verb(w!("taskbarunpin"), &id);
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
    update_settings(app, |s| {
        if !s.hidden_pinned.iter().any(|h| h.eq_ignore_ascii_case(&id)) {
            s.hidden_pinned.push(id.clone());
        }
    });
}

/// Does Windows list this program or Store app as pinned right now?
fn windows_has(exe: &str, aumid: &str) -> bool {
    if !aumid.is_empty() && scan_aumids(&favorites_blob()).iter().any(|(_, a)| a.eq_ignore_ascii_case(aumid)) {
        return true;
    }
    if exe.is_empty() {
        return false;
    }
    let dir = pinned_dir();
    let Ok(read) = std::fs::read_dir(&dir) else { return false };
    read.flatten().filter(|e| e.file_name().to_str().is_some_and(is_shortcut)).any(|e| shortcut_target(&e.path()).is_some_and(|t| norm(&t) == norm(exe)))
}

/// Pins a running app: Windows's own command first; when Windows does not pin it, MacDows keeps the pin itself
/// and the dock shows it all the same.
pub fn pin_app(app: &AppHandle, exe: &str, aumid: &str, name: &str) {
    let Some(source) = pin_source(exe, aumid) else { return };
    // A Store app is pinned (and later matched) by its app id, so the check must not use the host program's exe.
    let by_aumid = source.starts_with(APPS_FOLDER);
    let (check_exe, check_aumid) = if by_aumid { ("", aumid) } else { (exe, "") };
    if windows_has(check_exe, check_aumid) {
        return;
    }
    let accepted = windows_verb(w!("taskbarpin"), &source);
    for _ in 0..6 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if windows_has(check_exe, check_aumid) {
            eprintln!("[pinned] {source}: pinned through Windows");
            let _ = app.emit("pinned-changed", ());
            return;
        }
    }
    eprintln!("[pinned] {source}: Windows did not pin it (command accepted: {accepted}); kept by MacDows");
    let key = if by_aumid { aumid.to_lowercase() } else { norm(exe) };
    let local = crate::settings::LocalPin { id: format!("{LOCAL_PREFIX}{key}"), name: name.to_string(), exe: if by_aumid { String::new() } else { exe.to_string() }, aumid: if by_aumid { aumid.to_string() } else { String::new() } };
    update_settings(app, |s| {
        if !s.dock.local_pins.iter().any(|l| l.id.eq_ignore_ascii_case(&local.id)) {
            s.dock.local_pins.push(local.clone());
        }
    });
}

/// Changes the settings, saves them and tells the windows (the dock reloads its pins).
pub fn update_settings(app: &AppHandle, change: impl FnOnce(&mut crate::settings::Settings)) {
    let state = app.state::<crate::AppSettings>();
    let mut guard = state.0.lock().unwrap();
    change(&mut guard);
    let _ = crate::settings::save(&guard);
    let updated = guard.clone();
    drop(guard);
    let _ = app.emit("settings-changed", updated);
    let _ = app.emit("pinned-changed", ());
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

    fn app(id: &str, exe: &str, aumid: &str) -> PinnedApp {
        PinnedApp { id: id.into(), name: id.into(), exe: exe.into(), aumid: aumid.into(), icon_png_base64: None }
    }

    #[test]
    fn local_pins_follow_the_windows_pins() {
        let local = vec![crate::settings::LocalPin { id: "local:calc".into(), name: "Calc".into(), exe: "C:/calc.exe".into(), aumid: String::new() }];
        let merged = merge_local(vec![app("a", "C:/a.exe", "")], &local);
        assert_eq!(merged.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["a", "local:calc"]);
    }

    #[test]
    fn a_local_pin_windows_now_has_is_not_listed_twice() {
        let local = vec![crate::settings::LocalPin { id: "local:calc".into(), name: "Calc".into(), exe: r"c:\CALC.exe".into(), aumid: String::new() }];
        let merged = merge_local(vec![app("a", "C:/calc.exe", "")], &local);
        assert_eq!(merged.len(), 1);
    }

    #[test]
    fn apps_match_by_program_or_by_app_id() {
        assert!(same_app(&app("a", "C:/x/Edge.exe", ""), r"c:\x\edge.exe", ""));
        assert!(same_app(&app("s", "", "Pkg_8wekyb3d8bbwe!App"), "", "pkg_8wekyb3d8bbwe!app"));
        assert!(!same_app(&app("a", "C:/a.exe", ""), "", ""));
    }

    #[test]
    fn launching_uses_the_pin_itself_or_the_exe_or_the_app_id() {
        assert_eq!(launch_target("C:/pins/Edge.lnk", "", ""), Some("C:/pins/Edge.lnk".to_string()));
        assert_eq!(launch_target("", "C:/x/app.exe", ""), Some("C:/x/app.exe".to_string()));
        assert_eq!(launch_target("local:x", "C:/x/app.exe", "Pkg_8wekyb3d8bbwe!App"), Some(r"shell:AppsFolder\Pkg_8wekyb3d8bbwe!App".to_string()));
        assert_eq!(launch_target("", "", ""), None);
    }

    #[test]
    fn a_store_apps_host_program_is_never_pinned() {
        assert_eq!(pin_source(r"C:\Windows\System32\ApplicationFrameHost.exe", "Pkg_8wekyb3d8bbwe!App"), Some(r"shell:AppsFolder\Pkg_8wekyb3d8bbwe!App".to_string()));
        assert_eq!(pin_source(r"C:\Apps\app.exe", "Pkg_8wekyb3d8bbwe!App"), Some(r"C:\Apps\app.exe".to_string()));
        assert_eq!(pin_source(r"C:\Windows\System32\ApplicationFrameHost.exe", ""), None);
        assert_eq!(pin_source("", ""), None);
    }

    #[test]
    fn apply_order_keeps_the_saved_order_and_appends_new_pins() {
        let pins = vec![app("a", "", ""), app("b", "", ""), app("c", "", "")];
        let ordered = apply_order(pins, &["c".to_string(), "a".to_string()]);
        assert_eq!(ordered.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["c", "a", "b"]);
    }

    #[test]
    fn apply_order_ignores_pins_that_are_gone() {
        let ordered = apply_order(vec![app("a", "", "")], &["gone".to_string(), "a".to_string()]);
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].id, "a");
    }

    #[test]
    fn apply_order_ignores_case_and_leaves_an_empty_order_alone() {
        let ordered = apply_order(vec![app("A", "", ""), app("B", "", "")], &["b".to_string()]);
        assert_eq!(ordered.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["B", "A"]);
        let same = apply_order(vec![app("A", "", ""), app("B", "", "")], &[]);
        assert_eq!(same.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["A", "B"]);
    }

    /// Run by hand (`cargo test pinning_through_windows -- --ignored --nocapture`): pins Character Map through
    /// the Windows command and unpins it again, to see whether Windows honors `taskbarpin` for other programs.
    #[test]
    #[ignore]
    fn pinning_through_windows() {
        let exe = r"C:\Windows\System32\charmap.exe";
        assert!(!windows_has(exe, ""), "Character Map is already pinned; unpin it first");
        let accepted = windows_verb(w!("taskbarpin"), exe);
        let mut pinned = false;
        for _ in 0..10 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if windows_has(exe, "") {
                pinned = true;
                break;
            }
        }
        println!("taskbarpin accepted: {accepted}, pinned by Windows: {pinned}");
        if pinned {
            let dir = pinned_dir();
            for e in std::fs::read_dir(&dir).unwrap().flatten() {
                if shortcut_target(&e.path()).is_some_and(|t| norm(&t) == norm(exe)) {
                    println!("unpin accepted: {}", windows_verb(w!("taskbarunpin"), &e.path().to_string_lossy()));
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("still pinned after cleanup: {}", windows_has(exe, ""));
        }
    }
}
