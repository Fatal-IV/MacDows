//! Windows notifications, read through `UserNotificationListener` so the notch and the notification center
//! can show them in MacDows's own design. Windows's own banners keep appearing; this only reads and removes.
//!
//! An unpackaged app gets no change events, so the list is polled and compared with the last one.

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::Foundation::Size;
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::UI::Notifications::Management::{UserNotificationListener, UserNotificationListenerAccessStatus};
use windows::UI::Notifications::{KnownNotificationBindings, NotificationKinds, UserNotification};

#[derive(Serialize, Clone, Debug, PartialEq)]
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
                *LIST.lock().unwrap() = current;
                if let Some(app) = APP.get() {
                    if changed {
                        let _ = app.emit("notifications-changed", ());
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
