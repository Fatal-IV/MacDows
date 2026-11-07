//! The island and the dock live in transparent windows that are as big as the island or dock can ever get (580x340 and
//! 900x120). Without further care the empty part of such a window swallows clicks meant for the program underneath.
//! The front end reports the real size of what it draws; while the pointer is outside that shape the window ignores
//! the mouse (clicks fall through to the windows below), inside it the window takes it. (Cutting the window with
//! `SetWindowRgn` did the same but painted the cut-out margin white.) The cursor only counts as "near" inside the shape.

use crate::fullscreen::Rect;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::Graphics::Gdi::SetWindowRgn;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

/// True when the point is inside the rectangle grown by `pad` pixels on every side.
pub fn point_in(r: Rect, x: i32, y: i32, pad: i32) -> bool {
    x >= r.left - pad && x < r.right + pad && y >= r.top - pad && y < r.bottom + pad
}

/// Where each bar's visible shape is on the screen, by window label.
static SCREEN_RECTS: Mutex<Option<HashMap<String, Rect>>> = Mutex::new(None);

/// The shape the front end asked for, kept inside the window (physical pixels, relative to the window's top left).
/// Returns `(left, top, right, bottom)`, or `None` when nothing of it is left inside the window.
pub fn clamp_to_window(x: i32, y: i32, w: i32, h: i32, win_w: i32, win_h: i32) -> Option<(i32, i32, i32, i32)> {
    let (left, top) = (x.max(0), y.max(0));
    let (right, bottom) = ((x.saturating_add(w)).min(win_w), (y.saturating_add(h)).min(win_h));
    (right > left && bottom > top).then_some((left, top, right, bottom))
}

fn is_bar(label: &str) -> bool {
    crate::bar::is_bar(label)
}

/// The visible shape of a bar in screen coordinates, when the front end has reported one.
pub fn screen_rect(label: &str) -> Option<Rect> {
    SCREEN_RECTS.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(label).copied()
}

/// `r` moved by (dx, dy).
pub fn shifted(r: Rect, dx: i32, dy: i32) -> Rect {
    Rect { left: r.left + dx, top: r.top + dy, right: r.right + dx, bottom: r.bottom + dy }
}

/// The window was moved on the screen: its remembered shape (in screen coordinates) moves along. Without this the shape stays where the
/// window used to be and the pointer is never "inside" it, so the bar ignores every click.
pub fn shift(label: &str, dx: i32, dy: i32) {
    if let Some(r) = screen_rect(label) {
        remember(label, Some(shifted(r, dx, dy)));
    }
}

/// Drops what is remembered about a bar window that was closed.
pub fn forget(label: &str) {
    remember(label, None);
}

fn remember(label: &str, rect: Option<Rect>) {
    let mut guard = SCREEN_RECTS.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(HashMap::new);
    match rect {
        Some(r) => {
            map.insert(label.to_string(), r);
        }
        None => {
            map.remove(label);
        }
    }
}

/// Cuts the bar's window down to a rounded rectangle (physical pixels, relative to the window).
#[tauri::command]
pub fn set_hit_region(app: AppHandle, label: String, x: i32, y: i32, w: i32, h: i32, radius: i32) -> Result<(), String> {
    if !is_bar(&label) {
        return Err("unknown window".into());
    }
    let win = app.get_webview_window(&label).ok_or("no such window")?;
    let size = win.inner_size().map_err(|e| e.to_string())?;
    let origin = win.outer_position().map_err(|e| e.to_string())?;
    let Some((left, top, right, bottom)) = clamp_to_window(x, y, w, h, size.width as i32, size.height as i32) else {
        return clear_hit_region(app, label);
    };
    let _ = radius;
    // No window region any more: it painted its margin white. An old region (there is none now) would be removed here.
    if let Ok(h) = win.hwnd() {
        unsafe {
            SetWindowRgn(HWND(h.0), None, false);
        }
    }
    remember(&label, Some(Rect { left: origin.x + left, top: origin.y + top, right: origin.x + right, bottom: origin.y + bottom }));
    Ok(())
}

/// Gives the bar its whole window back (and the whole window counts as the bar again).
#[tauri::command]
pub fn clear_hit_region(app: AppHandle, label: String) -> Result<(), String> {
    if !is_bar(&label) {
        return Err("unknown window".into());
    }
    let win = app.get_webview_window(&label).ok_or("no such window")?;
    let hwnd = HWND(win.hwnd().map_err(|e| e.to_string())?.0);
    unsafe {
        SetWindowRgn(hwnd, None, false);
    }
    remember(&label, None);
    Ok(())
}

/// What to do with a bar's "ignore the mouse" flag. A hidden window must not be touched at all: changing the flag
/// makes the window toolkit re-apply the window's state, which showed the dock again behind `visibility.rs`'s back
/// (it hides with a plain `ShowWindow`) every time the pointer crossed the dock's remembered shape, and then the dock
/// swallowed the clicks meant for the program underneath (Spotify's player buttons).
pub fn ignore_update(visible: bool, current: Option<bool>, want: bool) -> Option<bool> {
    if !visible || current == Some(want) {
        None
    } else {
        Some(want)
    }
}

/// Watches the pointer and lets each bar's window ignore the mouse while the pointer is outside its drawn shape.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut ignoring: HashMap<String, bool> = HashMap::new();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(16));
            let mut p = POINT::default();
            if unsafe { GetCursorPos(&mut p) }.is_err() {
                continue;
            }
            let labels = crate::bar::bar_labels(&app);
            ignoring.retain(|l, _| labels.contains(l));
            for label in labels.iter().map(String::as_str) {
                let want = match screen_rect(label) {
                    Some(r) => !point_in(r, p.x, p.y, 2),
                    None => false,
                };
                let Some(win) = app.get_webview_window(label) else { continue };
                let visible = win.is_visible().unwrap_or(false);
                if !visible {
                    // Applied again from scratch as soon as the bar is shown.
                    ignoring.remove(label);
                }
                if let Some(flag) = ignore_update(visible, ignoring.get(label).copied(), want) {
                    if win.set_ignore_cursor_events(flag).is_ok() {
                        ignoring.insert(label.to_string(), flag);
                    }
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_shape_inside_the_window_is_kept() {
        assert_eq!(clamp_to_window(100, 0, 200, 40, 580, 340), Some((100, 0, 300, 40)));
    }

    #[test]
    fn a_shape_sticking_out_is_cut_at_the_window_edges() {
        assert_eq!(clamp_to_window(-20, -8, 700, 400, 580, 340), Some((0, 0, 580, 340)));
    }

    #[test]
    fn a_shape_outside_the_window_or_empty_leaves_nothing() {
        assert_eq!(clamp_to_window(600, 0, 50, 50, 580, 340), None);
        assert_eq!(clamp_to_window(10, 10, 0, 40, 580, 340), None);
        assert_eq!(clamp_to_window(10, 10, 40, -5, 580, 340), None);
    }

    #[test]
    fn the_pointer_is_inside_a_shape_up_to_its_edge() {
        let r = Rect { left: 10, top: 0, right: 110, bottom: 40 };
        assert!(point_in(r, 10, 0, 0) && point_in(r, 109, 39, 0));
        assert!(!point_in(r, 110, 20, 0) && !point_in(r, 9, 20, 0));
        assert!(point_in(r, 110, 20, 2) && point_in(r, 8, 20, 2));
    }

    #[test]
    fn a_hidden_bar_is_never_touched() {
        assert_eq!(ignore_update(false, None, true), None);
        assert_eq!(ignore_update(false, Some(true), false), None);
    }

    #[test]
    fn a_shown_bar_only_changes_when_the_flag_differs() {
        assert_eq!(ignore_update(true, None, false), Some(false));
        assert_eq!(ignore_update(true, Some(false), false), None);
        assert_eq!(ignore_update(true, Some(false), true), Some(true));
    }

    #[test]
    fn only_the_two_bars_can_be_shaped() {
        assert!(is_bar("notch") && is_bar("dock"));
        assert!(is_bar("notch-2") && is_bar("dock-3"), "the bars of another monitor");
        assert!(!is_bar("settings") && !is_bar("tray") && !is_bar("preview"));
    }

    #[test]
    fn the_reported_shape_is_remembered_and_forgotten() {
        remember("test-bar", Some(Rect { left: 1, top: 2, right: 3, bottom: 4 }));
        assert_eq!(screen_rect("test-bar").map(|r| (r.left, r.top, r.right, r.bottom)), Some((1, 2, 3, 4)));
        remember("test-bar", None);
        assert!(screen_rect("test-bar").is_none());
    }

    #[test]
    fn a_remembered_shape_moves_with_its_window() {
        let r = Rect { left: 100, top: 0, right: 300, bottom: 40 };
        let moved = shifted(r, 40, 0);
        assert_eq!((moved.left, moved.top, moved.right, moved.bottom), (140, 0, 340, 40));
        remember("move-bar", Some(r));
        shift("move-bar", -20, 5);
        assert_eq!(screen_rect("move-bar").map(|r| (r.left, r.top, r.right, r.bottom)), Some((80, 5, 280, 45)));
        shift("never-reported", 1, 1);
        remember("move-bar", None);
    }
}
