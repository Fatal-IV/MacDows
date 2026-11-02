use crate::fullscreen::{self, should_hide, Rect};
use crate::settings::Visibility;
use crate::{windows_list, AppSettings};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetWindowRect, ShowWindow, SW_HIDE, SW_SHOWNOACTIVATE};

#[derive(Debug, Clone, Copy)]
pub struct Inputs {
    pub vis: Visibility,
    pub fullscreen: Option<bool>,
    pub overlapped: bool,
    pub cursor_near: bool,
}

pub fn rect_intersects(a: Rect, b: Rect) -> bool {
    a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom
}

pub fn point_in_rect(x: i32, y: i32, r: Rect) -> bool {
    x >= r.left && x < r.right && y >= r.top && y < r.bottom
}

/// Whether a bar (notch or dock) should be shown right now.
pub fn is_visible(i: Inputs) -> bool {
    if should_hide(i.vis, i.fullscreen) {
        return false;
    }
    match i.vis {
        Visibility::Fixed => true,
        Visibility::Smart => !i.overlapped || i.cursor_near,
        Visibility::Peek => i.cursor_near,
    }
}

fn rect_of(hwnd: HWND) -> Option<Rect> {
    let mut r = RECT::default();
    unsafe { GetWindowRect(hwnd, &mut r).ok()? };
    Some(Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom })
}

fn cursor() -> Option<(i32, i32)> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p).ok()? };
    Some((p.x, p.y))
}

/// The part of the screen another window must cover to count as "overlapping" the bar.
fn overlap_probe(label: &str, w: Rect) -> Rect {
    let cx = (w.left + w.right) / 2;
    if label == "notch" {
        Rect { left: cx - 85, top: w.top, right: cx + 85, bottom: w.top + 32 }
    } else {
        Rect { left: cx - 250, top: w.bottom - 80, right: cx + 250, bottom: w.bottom }
    }
}

/// Thin band where the cursor reveals a hidden bar.
fn reveal_band(label: &str, w: Rect) -> Rect {
    if label == "notch" {
        Rect { left: w.left, top: w.top, right: w.right, bottom: w.top + 3 }
    } else {
        Rect { left: w.left, top: w.bottom - 4, right: w.right, bottom: w.bottom + 4 }
    }
}

/// Applies visibility modes to the `notch` and `dock` windows. Cursor and window geometry are
/// sampled every 100 ms, and every 500 ms when both bars are Fixed.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut shown: [Option<bool>; 2] = [None, None];
        loop {
            let s = app.state::<AppSettings>().0.lock().unwrap().clone();
            let both_fixed = s.notch_visibility == Visibility::Fixed && s.dock_visibility == Visibility::Fixed;
            std::thread::sleep(Duration::from_millis(if both_fixed { 500 } else { 100 }));
            let fs = fullscreen::state();
            let cur = cursor();
            let needs_overlap = s.notch_visibility == Visibility::Smart || s.dock_visibility == Visibility::Smart;
            let others = if needs_overlap { windows_list::visible_rects() } else { Vec::new() };
            for (i, (label, vis)) in [("notch", s.notch_visibility), ("dock", s.dock_visibility)].into_iter().enumerate() {
                let Some(win) = app.get_webview_window(label) else { continue };
                let Ok(hwnd) = win.hwnd() else { continue };
                let hwnd = HWND(hwnd.0);
                let Some(r) = rect_of(hwnd) else { continue };
                let zone = if shown[i].unwrap_or(true) { r } else { reveal_band(label, r) };
                let near = cur.map(|(x, y)| point_in_rect(x, y, zone)).unwrap_or(false);
                let probe = overlap_probe(label, r);
                let overlapped = others.iter().any(|o| rect_intersects(*o, probe));
                let want = is_visible(Inputs { vis, fullscreen: fs, overlapped, cursor_near: near });
                if shown[i] != Some(want) {
                    unsafe {
                        let _ = ShowWindow(hwnd, if want { SW_SHOWNOACTIVATE } else { SW_HIDE });
                    }
                    shown[i] = Some(want);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fullscreen::Rect;
    use crate::settings::Visibility::{Fixed, Peek, Smart};

    fn inp(vis: Visibility, fullscreen: Option<bool>, overlapped: bool, cursor_near: bool) -> Inputs {
        Inputs { vis, fullscreen, overlapped, cursor_near }
    }

    #[test]
    fn fixed_is_always_visible() {
        assert!(is_visible(inp(Fixed, Some(true), true, false)));
        assert!(is_visible(inp(Fixed, None, true, false)));
    }

    #[test]
    fn fullscreen_hides_smart_and_peek_even_with_cursor_near() {
        assert!(!is_visible(inp(Smart, Some(true), false, true)));
        assert!(!is_visible(inp(Peek, Some(true), false, true)));
    }

    #[test]
    fn smart_hides_only_when_overlapped_and_cursor_away() {
        assert!(is_visible(inp(Smart, Some(false), false, false)));
        assert!(!is_visible(inp(Smart, Some(false), true, false)));
        assert!(is_visible(inp(Smart, Some(false), true, true)));
    }

    #[test]
    fn peek_shows_only_when_cursor_near() {
        assert!(!is_visible(inp(Peek, Some(false), false, false)));
        assert!(is_visible(inp(Peek, Some(false), false, true)));
    }

    #[test]
    fn rects_intersect_only_when_they_overlap() {
        let a = Rect { left: 0, top: 0, right: 100, bottom: 100 };
        assert!(rect_intersects(a, Rect { left: 50, top: 50, right: 150, bottom: 150 }));
        assert!(!rect_intersects(a, Rect { left: 100, top: 0, right: 200, bottom: 100 }));
        assert!(!rect_intersects(a, Rect { left: 0, top: 200, right: 100, bottom: 300 }));
    }

    #[test]
    fn point_in_rect_is_inclusive_of_edges_that_belong_to_it() {
        let r = Rect { left: 10, top: 10, right: 20, bottom: 20 };
        assert!(point_in_rect(10, 10, r));
        assert!(!point_in_rect(20, 20, r));
    }
}
