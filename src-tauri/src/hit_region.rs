//! The island and the dock live in transparent windows that are as big as the island or dock can ever get (580x340 and
//! 900x120). Without further care the empty part of such a window swallows clicks meant for the program underneath.
//! The front end reports the real size of what it draws and the window is cut down to that shape (`SetWindowRgn`),
//! so everything outside it belongs to the windows below, and the cursor only counts as "near" inside it.

use crate::fullscreen::Rect;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{CreateRoundRectRgn, GetStockObject, SetWindowRgn, NULL_BRUSH};
use windows::Win32::UI::WindowsAndMessaging::{SetClassLongPtrW, GCLP_HBRBACKGROUND};

/// Windows paints the area a new region exposes with the window class's background brush (white) until the page has
/// painted over it: a white bar flashed under the island and the dock whenever their shape grew. No brush, no flash.
unsafe fn no_background(hwnd: HWND) {
    SetClassLongPtrW(hwnd, GCLP_HBRBACKGROUND, GetStockObject(NULL_BRUSH).0 as isize);
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
    matches!(label, "notch" | "dock")
}

/// The visible shape of a bar in screen coordinates, when the front end has reported one.
pub fn screen_rect(label: &str) -> Option<Rect> {
    SCREEN_RECTS.lock().unwrap_or_else(|e| e.into_inner()).as_ref()?.get(label).copied()
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
    let hwnd = HWND(win.hwnd().map_err(|e| e.to_string())?.0);
    let d = radius.max(0).saturating_mul(2);
    unsafe {
        // The window owns the region once SetWindowRgn accepts it.
        let region = CreateRoundRectRgn(left, top, right + 1, bottom + 1, d, d);
        no_background(hwnd);
        SetWindowRgn(hwnd, Some(region), false);
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
        no_background(hwnd);
        SetWindowRgn(hwnd, None, false);
    }
    remember(&label, None);
    Ok(())
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
    fn only_the_two_bars_can_be_shaped() {
        assert!(is_bar("notch") && is_bar("dock"));
        assert!(!is_bar("settings") && !is_bar("tray") && !is_bar("preview"));
    }

    #[test]
    fn the_reported_shape_is_remembered_and_forgotten() {
        remember("test-bar", Some(Rect { left: 1, top: 2, right: 3, bottom: 4 }));
        assert_eq!(screen_rect("test-bar").map(|r| (r.left, r.top, r.right, r.bottom)), Some((1, 2, 3, 4)));
        remember("test-bar", None);
        assert!(screen_rect("test-bar").is_none());
    }
}
