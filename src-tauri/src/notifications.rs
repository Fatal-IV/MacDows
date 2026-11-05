//! Windows notifications, read through `UserNotificationListener` so the notch and the notification center
//! can show them in MacDows's own design. Windows's own banners keep appearing; this only reads and removes.
//!
//! An unpackaged app gets no change events, so the list is polled and compared with the last one.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::Foundation::Size;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::UI::Notifications::Management::{UserNotificationListener, UserNotificationListenerAccessStatus};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds, UserNotification};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Notice {
    pub id: u32,
    pub app: String,
    pub title: String,
    pub body: String,
    /// Unix time in milliseconds.
    pub time_ms: i64,
    pub icon_png_base64: Option<String>,
}

/// Windows counts 100 ns ticks from 1601; the web side wants Unix milliseconds.
pub fn unix_ms(universal_time: i64) -> i64 {
    universal_time / 10_000 - 11_644_473_600_000
}

/// Newest first, which is how every list in the UI reads.
pub fn sort_newest_first(list: &mut [Notice]) {
    list.sort_by(|a, b| b.time_ms.cmp(&a.time_ms).then(b.id.cmp(&a.id)));
}

/// The notifications that were not in `known` yet. On the very first look everything is "known":
/// old notifications must not pop up again when MacDows starts.
pub fn fresh(known: &Option<HashSet<u32>>, current: &[Notice]) -> Vec<Notice> {
    match known {
        None => Vec::new(),
        Some(set) => current.iter().filter(|n| !set.contains(&n.id)).cloned().collect(),
    }
}

/// How long the notch stays in view for a banner: a little longer than the banner itself.
const BANNER_HOLD_MS: u64 = 7_000;

/// How many past notifications are kept.
const HISTORY_MAX: usize = 100;

/// A notification is "the same one" when app, title, body and time match (Windows reuses ids between sessions).
fn same(a: &Notice, b: &Notice) -> bool {
    a.time_ms == b.time_ms && a.app == b.app && a.title == b.title && a.body == b.body
}

/// `incoming` joins the history; ones already in it are skipped. Newest first, capped at `max`.
/// Returns whether anything was added.
pub fn merge_history(history: &mut Vec<Notice>, incoming: &[Notice], max: usize) -> bool {
    let mut added = false;
    for n in incoming {
        if !history.iter().any(|h| same(h, n)) {
            history.push(n.clone());
            added = true;
        }
    }
    if added {
        sort_newest_first(history);
        history.truncate(max);
    }
    added
}

/// Only what arrived after `cutoff` (the moment the history was last cleared) may join the history: Windows keeps
/// the old notifications in its own list, and without this they would flow straight back in after a clear.
pub fn after_cutoff(list: &[Notice], cutoff: i64) -> Vec<Notice> {
    list.iter().filter(|n| n.time_ms > cutoff).cloned().collect()
}

fn cutoff_path() -> std::path::PathBuf {
    history_path().with_file_name("notification-history-cleared-at")
}

/// When the history was last cleared (Unix ms); 0 when it never was. Kept on disk so a restart remembers it.
fn cleared_at() -> i64 {
    *CLEARED_AT.lock().unwrap().get_or_insert_with(|| std::fs::read_to_string(cutoff_path()).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0))
}

static CLEARED_AT: Mutex<Option<i64>> = Mutex::new(None);

fn mark_cleared() {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0);
    let _ = std::fs::create_dir_all(cutoff_path().parent().unwrap_or(std::path::Path::new(".")));
    let _ = std::fs::write(cutoff_path(), now.to_string());
    *CLEARED_AT.lock().unwrap() = Some(now);
}

fn history_path() -> std::path::PathBuf {
    std::env::var_os("APPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir).join("MacDows").join("notification-history.json")
}

static HISTORY: Mutex<Option<Vec<Notice>>> = Mutex::new(None);

/// The history file is encrypted for the current Windows user (see `secure_store`). A plain file from an earlier
/// version is still read, and replaced by an encrypted one the first time it is loaded.
fn load_history() -> Vec<Notice> {
    let Ok(bytes) = std::fs::read(history_path()) else { return Vec::new() };
    if crate::secure_store::is_sealed(&bytes) {
        return crate::secure_store::open(&bytes).and_then(|plain| serde_json::from_slice(&plain).ok()).unwrap_or_default();
    }
    let list: Vec<Notice> = serde_json::from_slice(&bytes).unwrap_or_default();
    save_history(&list);
    list
}

fn with_history<R>(f: impl FnOnce(&mut Vec<Notice>) -> R) -> R {
    let mut guard = HISTORY.lock().unwrap();
    let list = guard.get_or_insert_with(load_history);
    f(list)
}

fn save_history(list: &[Notice]) {
    let path = history_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    // Never written in plain text: if Windows cannot encrypt, the file is removed instead.
    match serde_json::to_vec(list).ok().and_then(|plain| crate::secure_store::seal(&plain)) {
        Some(sealed) => {
            let _ = std::fs::write(path, sealed);
        }
        None => {
            let _ = std::fs::remove_file(path);
        }
    }
}

static APP: OnceLock<AppHandle> = OnceLock::new();
static LIST: Mutex<Vec<Notice>> = Mutex::new(Vec::new());

fn read_notice(n: &UserNotification, icons: &mut HashMap<String, Option<String>>) -> Option<Notice> {
    use base64::Engine;
    let id = n.Id().ok()?;
    let info = n.AppInfo().ok()?.DisplayInfo().ok()?;
    let app = info.DisplayName().map(|s| s.to_string()).unwrap_or_default();
    let binding = n.Notification().ok()?.Visual().ok()?.GetBinding(&KnownNotificationBindings::ToastGeneric().ok()?).ok()?;
    let texts: Vec<String> = binding.GetTextElements().ok()?.into_iter().filter_map(|t| t.Text().ok()).map(|s| s.to_string()).collect();
    let title = texts.first().cloned().unwrap_or_default();
    let body = texts.iter().skip(1).cloned().collect::<Vec<_>>().join("\n");
    let time_ms = n.CreationTime().map(|t| unix_ms(t.UniversalTime)).unwrap_or(0);
    let icon = icons
        .entry(app.clone())
        .or_insert_with(|| {
            let logo = info.GetLogo(Size { Width: 48.0, Height: 48.0 }).ok()?;
            let bytes = crate::media::read_thumbnail(logo.into())?;
            Some(base64::engine::general_purpose::STANDARD.encode(bytes))
        })
        .clone();
    Some(Notice { id, app, title, body, time_ms, icon_png_base64: icon })
}

fn snapshot(listener: &UserNotificationListener, icons: &mut HashMap<String, Option<String>>) -> Option<Vec<Notice>> {
    let list = listener.GetNotificationsAsync(NotificationKinds::Toast).ok()?.get().ok()?;
    let mut out: Vec<Notice> = list.into_iter().filter_map(|n| read_notice(&n, icons)).collect();
    sort_newest_first(&mut out);
    Some(out)
}

pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    std::thread::spawn(|| {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok(listener) = UserNotificationListener::Current() else { return };
        if listener.GetAccessStatus().ok() != Some(UserNotificationListenerAccessStatus::Allowed) {
            let allowed = listener.RequestAccessAsync().and_then(|op| op.get()).ok() == Some(UserNotificationListenerAccessStatus::Allowed);
            if !allowed {
                return;
            }
        }
        let mut icons = HashMap::new();
        let mut known: Option<HashSet<u32>> = None;
        loop {
            if let Some(current) = snapshot(&listener, &mut icons) {
                let ids: HashSet<u32> = current.iter().map(|n| n.id).collect();
                let changed = known.as_ref() != Some(&ids);
                let new = fresh(&known, &current);
                with_history(|h| {
                    if merge_history(h, &after_cutoff(&current, cleared_at()), HISTORY_MAX) {
                        save_history(h);
                    }
                });
                *LIST.lock().unwrap() = current;
                if let Some(app) = APP.get() {
                    if changed {
                        let _ = app.emit("notifications-changed", ());
                        let _ = app.emit("notification-history-changed", ());
                    }
                    // The newest one is the one the notch shows.
                    if let Some(n) = new.first() {
                        crate::visibility::hold_notch(BANNER_HOLD_MS);
                        let _ = app.emit("notification-new", n);
                    }
                }
                known = Some(ids);
            }
            std::thread::sleep(Duration::from_millis(1500));
        }
    });
}

#[tauri::command]
pub fn get_notifications() -> Vec<Notice> {
    LIST.lock().unwrap().clone()
}

/// Every notification seen so far (also the ones dismissed in Windows), newest first.
#[tauri::command]
pub fn get_notification_history() -> Vec<Notice> {
    with_history(|h| h.clone())
}

#[tauri::command]
pub fn clear_notification_history() {
    mark_cleared();
    with_history(|h| {
        h.clear();
        save_history(h);
    });
}

#[tauri::command]
pub async fn remove_notification(id: u32) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        UserNotificationListener::Current().and_then(|l| l.RemoveNotification(id)).map_err(|e| e.message())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn clear_notifications() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(|| UserNotificationListener::Current().and_then(|l| l.ClearNotifications()).map_err(|e| e.message()))
        .await
        .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: u32, time_ms: i64) -> Notice {
        Notice { id, app: "App".into(), title: "T".into(), body: "B".into(), time_ms, icon_png_base64: None }
    }

    #[test]
    fn windows_ticks_become_unix_milliseconds() {
        // 1970-01-01 in Windows ticks
        assert_eq!(unix_ms(116_444_736_000_000_000), 0);
        assert_eq!(unix_ms(116_444_736_010_000_000), 1_000);
    }

    #[test]
    fn newest_comes_first() {
        let mut list = vec![n(1, 100), n(2, 300), n(3, 200)];
        sort_newest_first(&mut list);
        assert_eq!(list.iter().map(|x| x.id).collect::<Vec<_>>(), vec![2, 3, 1]);
    }

    #[test]
    fn history_skips_what_it_already_has_and_keeps_newest_first() {
        let mut h = vec![n(1, 100)];
        assert!(merge_history(&mut h, &[n(1, 100), n(2, 300)], 10));
        assert_eq!(h.iter().map(|x| x.time_ms).collect::<Vec<_>>(), vec![300, 100]);
        assert!(!merge_history(&mut h, &[n(2, 300)], 10));
    }

    #[test]
    fn a_cleared_history_does_not_take_the_old_ones_back() {
        let live = [n(1, 100), n(2, 300)];
        assert_eq!(after_cutoff(&live, 300).len(), 0);
        assert_eq!(after_cutoff(&live, 200).iter().map(|x| x.id).collect::<Vec<_>>(), vec![2]);
        assert_eq!(after_cutoff(&live, 0).len(), 2);
    }

    #[test]
    fn history_is_capped() {
        let mut h = Vec::new();
        merge_history(&mut h, &[n(1, 1), n(2, 2), n(3, 3)], 2);
        assert_eq!(h.iter().map(|x| x.time_ms).collect::<Vec<_>>(), vec![3, 2]);
    }

    #[test]
    fn nothing_is_fresh_on_the_first_look() {
        assert!(fresh(&None, &[n(1, 1), n(2, 2)]).is_empty());
    }

    #[test]
    fn only_unseen_ids_are_fresh() {
        let known = Some(HashSet::from([1, 2]));
        let got = fresh(&known, &[n(3, 30), n(2, 20), n(1, 10)]);
        assert_eq!(got.iter().map(|x| x.id).collect::<Vec<_>>(), vec![3]);
    }
}
