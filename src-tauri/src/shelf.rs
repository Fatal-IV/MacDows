//! The shelf of the notch: files dropped on it are remembered (by path, nothing is copied) so they can be
//! opened, revealed, or dragged on to another app later.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

const MAX_ITEMS: usize = 40;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct ShelfItem {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    /// The file was moved or deleted since it was put on the shelf.
    pub missing: bool,
    pub icon_png_base64: Option<String>,
    /// A PNG file with the icon, for the drag preview when the item is dragged out.
    pub icon_path: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
struct Stored {
    /// The Windows logon session the shelf belongs to; a new session (after log-off, restart or shutdown) starts with an empty shelf.
    #[serde(default)]
    session: String,
    #[serde(default)]
    paths: Vec<String>,
}

/// The shelf is kept while the same logon session goes on, and dropped for any other one.
pub fn same_session(stored: &str, current: &str) -> bool {
    !current.is_empty() && stored == current
}

/// An identifier of the current logon session (the logon id of this process's token); empty if it cannot be read.
fn current_session() -> String {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{GetTokenInformation, TokenStatistics, TOKEN_QUERY, TOKEN_STATISTICS};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return String::new();
        }
        let mut stats = TOKEN_STATISTICS::default();
        let mut len = 0u32;
        let ok = GetTokenInformation(token, TokenStatistics, Some(&mut stats as *mut _ as *mut _), std::mem::size_of::<TOKEN_STATISTICS>() as u32, &mut len).is_ok();
        let _ = CloseHandle(token);
        if ok { format!("{:x}-{:x}", stats.AuthenticationId.HighPart, stats.AuthenticationId.LowPart) } else { String::new() }
    }
}

/// `new` goes to the front (newest first); a path already on the shelf moves up instead of repeating; the shelf keeps `max` items.
pub fn merge(existing: &[String], new: &[String], max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in new.iter().chain(existing.iter()) {
        if !out.iter().any(|o| o.eq_ignore_ascii_case(p)) {
            out.push(p.clone());
        }
    }
    out.truncate(max);
    out
}

pub fn file_name(path: &str) -> String {
    Path::new(path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.to_string())
}

fn dir() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("MacDows")
}

fn load() -> Vec<String> {
    let Some(stored) = std::fs::read_to_string(dir().join("shelf.json")).ok().and_then(|t| serde_json::from_str::<Stored>(&t).ok()) else { return Vec::new() };
    if same_session(&stored.session, &current_session()) {
        return stored.paths;
    }
    // The old session is over: forget its files and their icons.
    let _ = std::fs::remove_file(dir().join("shelf.json"));
    let _ = std::fs::remove_dir_all(dir().join("shelf-icons"));
    Vec::new()
}

fn save(paths: &[String]) {
    let _ = std::fs::create_dir_all(dir());
    if let Ok(text) = serde_json::to_string(&Stored { session: current_session(), paths: paths.to_vec() }) {
        let _ = std::fs::write(dir().join("shelf.json"), text);
    }
}

static ICONS: Mutex<Option<HashMap<String, (Option<String>, Option<String>)>>> = Mutex::new(None);

/// Icon of the file as base64 and as a PNG file on disk (the drag preview needs a file).
fn icon_for(path: &str) -> (Option<String>, Option<String>) {
    use base64::Engine;
    let mut guard = ICONS.lock().unwrap();
    let cache = guard.get_or_insert_with(HashMap::new);
    if let Some(hit) = cache.get(path) {
        return hit.clone();
    }
    let b64 = crate::windows_list::icon_png_base64(path);
    let file = b64.as_ref().and_then(|b| {
        let bytes = base64::engine::general_purpose::STANDARD.decode(b).ok()?;
        let folder = dir().join("shelf-icons");
        std::fs::create_dir_all(&folder).ok()?;
        let name: String = path.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        let name: String = name.chars().rev().take(80).collect::<Vec<_>>().into_iter().rev().collect();
        let file = folder.join(format!("{name}.png"));
        std::fs::write(&file, bytes).ok()?;
        Some(file.to_string_lossy().into_owned())
    });
    cache.insert(path.to_string(), (b64.clone(), file.clone()));
    (b64, file)
}

fn items(paths: &[String]) -> Vec<ShelfItem> {
    paths
        .iter()
        .map(|p| {
            let meta = std::fs::metadata(p);
            let (icon_png_base64, icon_path) = if meta.is_ok() { icon_for(p) } else { (None, None) };
            ShelfItem { path: p.clone(), name: file_name(p), is_dir: meta.as_ref().map(|m| m.is_dir()).unwrap_or(false), missing: meta.is_err(), icon_png_base64, icon_path }
        })
        .collect()
}

#[tauri::command]
pub async fn get_shelf() -> Vec<ShelfItem> {
    tauri::async_runtime::spawn_blocking(|| items(&load())).await.unwrap_or_default()
}

#[tauri::command]
pub async fn add_to_shelf(paths: Vec<String>) -> Vec<ShelfItem> {
    tauri::async_runtime::spawn_blocking(move || {
        let merged = merge(&load(), &paths, MAX_ITEMS);
        save(&merged);
        items(&merged)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn remove_from_shelf(path: String) -> Vec<ShelfItem> {
    tauri::async_runtime::spawn_blocking(move || {
        let kept: Vec<String> = load().into_iter().filter(|p| !p.eq_ignore_ascii_case(&path)).collect();
        save(&kept);
        items(&kept)
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn clear_shelf() {
    let _ = tauri::async_runtime::spawn_blocking(|| save(&[])).await;
}

/// Only paths that are on the shelf may be opened or revealed from the web side.
fn on_shelf(path: &str) -> bool {
    load().iter().any(|p| p.eq_ignore_ascii_case(path))
}

#[tauri::command]
pub fn open_shelf_item(path: String) -> Result<(), String> {
    if !on_shelf(&path) {
        return Err("not on the shelf".into());
    }
    Command::new("explorer.exe").arg(&path).creation_flags(CREATE_NO_WINDOW).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn reveal_shelf_item(path: String) -> Result<(), String> {
    if !on_shelf(&path) {
        return Err("not on the shelf".into());
    }
    Command::new("explorer.exe").arg(format!("/select,{path}")).creation_flags(CREATE_NO_WINDOW).spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn the_shelf_only_lives_within_one_logon_session() {
        assert!(same_session("0-3e7abc", "0-3e7abc"));
        assert!(!same_session("0-3e7abc", "0-3e7abd"));
        assert!(!same_session("", ""));
        assert!(!same_session("0-3e7abc", ""));
    }

    #[test]
    fn this_process_has_a_session_id() {
        assert!(!current_session().is_empty());
    }

    #[test]
    fn new_files_go_first() {
        assert_eq!(merge(&v(&["a", "b"]), &v(&["c"]), 10), v(&["c", "a", "b"]));
    }

    #[test]
    fn a_repeated_path_moves_up_instead_of_doubling() {
        assert_eq!(merge(&v(&["a", "b", "c"]), &v(&["c"]), 10), v(&["c", "a", "b"]));
        assert_eq!(merge(&v(&["C:\\x\\A.txt"]), &v(&["c:\\x\\a.TXT"]), 10).len(), 1);
    }

    #[test]
    fn the_shelf_is_capped_and_drops_the_oldest() {
        assert_eq!(merge(&v(&["a", "b", "c"]), &v(&["d"]), 3), v(&["d", "a", "b"]));
    }

    #[test]
    fn names_come_from_the_last_path_part() {
        assert_eq!(file_name(r"C:\Users\x\Report final.pdf"), "Report final.pdf");
        assert_eq!(file_name(r"C:\Users\x\Folder"), "Folder");
    }
}
