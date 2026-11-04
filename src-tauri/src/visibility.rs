use crate::fullscreen::{self, should_hide, Rect};
use crate::island_settings::FullscreenMode;
use crate::settings::Visibility;
use crate::{windows_list, AppSettings};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Win32::Foundation::{CloseHandle, HWND, POINT, RECT};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::core::PWSTR;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, ShowWindow, SW_HIDE, SW_SHOWNOACTIVATE};

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
    // A fullscreen app never hides the bar for good: it only stays out of the way until the
    // cursor reaches the screen edge, like Peek.
    if should_hide(i.vis, i.fullscreen) {
        return i.cursor_near;
    }
    match i.vis {
        Visibility::Fixed => true,
        Visibility::Smart => !i.overlapped || i.cursor_near,
        Visibility::Peek => i.cursor_near,
    }
}

/// The island stays out of fullscreen apps altogether when the user chose so (no edge reveal, no banner); the dock
/// always keeps its edge reveal.
pub fn hidden_in_fullscreen(label: &str, mode: FullscreenMode, fullscreen: Option<bool>) -> bool {
    label == "notch" && mode == FullscreenMode::Hidden && fullscreen == Some(true)
}

/// The programs behind Windows's own Start, Search and quick settings.
pub fn is_shell_menu(exe: &str) -> bool {
    matches!(exe.to_ascii_lowercase().as_str(), "startmenuexperiencehost.exe" | "searchhost.exe" | "shellexperiencehost.exe")
}

static LAST_SHELL_MENU_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

static NOTCH_HOLD_UNTIL_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Keeps the notch on screen for `ms` milliseconds even when it would be hidden (a notification banner is showing).
pub fn hold_notch(ms: u64) {
    NOTCH_HOLD_UNTIL_MS.store(now_ms() + ms, std::sync::atomic::Ordering::Relaxed);
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// True when one of Windows's own menus was in front a moment ago but is not any more. Clicking a dock button takes
/// focus and closes the open menu before the click is handled; the click must then only close, not reopen.
pub fn shell_menu_just_closed() -> bool {
    let last = LAST_SHELL_MENU_MS.load(std::sync::atomic::Ordering::Relaxed);
    last != 0 && now_ms().saturating_sub(last) < 700 && !foreground_exe().is_some_and(|exe| is_shell_menu(&exe))
}

/// File name of the program that owns the window in front.
fn foreground_exe() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let result = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(process);
        result.ok()?;
        String::from_utf16_lossy(&buf[..len as usize]).rsplit('\\').next().map(str::to_string)
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
/// sampled every 100 ms, and every 250 ms when both bars are Fixed.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut shown: [Option<bool>; 2] = [None, None];
        let started = std::time::Instant::now();
        loop {
            let s = app.state::<AppSettings>().0.lock().unwrap().clone();
            let both_fixed = s.island.visibility == Visibility::Fixed && s.dock_visibility == Visibility::Fixed;
            std::thread::sleep(Duration::from_millis(if both_fixed { 250 } else { 100 }));
            let fs = fullscreen::state();
            let cur = cursor();
            let needs_overlap = s.island.visibility == Visibility::Smart || s.dock_visibility == Visibility::Smart;
            let others = if needs_overlap { windows_list::visible_rects() } else { Vec::new() };
            // While one of Windows's own menus is open (Start, Search, quick settings) the dock stays in view under it.
            let shell_menu_open = foreground_exe().is_some_and(|exe| is_shell_menu(&exe));
            if shell_menu_open {
                LAST_SHELL_MENU_MS.store(now_ms(), std::sync::atomic::Ordering::Relaxed);
            }
            for (i, (label, vis)) in [("notch", s.island.visibility), ("dock", s.dock_visibility)].into_iter().enumerate() {
                let Some(win) = app.get_webview_window(label) else { continue };
                let Ok(hwnd) = win.hwnd() else { continue };
                let hwnd = HWND(hwnd.0);
                let Some(r) = rect_of(hwnd) else { continue };
                let zone = if shown[i].unwrap_or(true) { r } else { reveal_band(label, r) };
                let near = cur.map(|(x, y)| point_in_rect(x, y, zone)).unwrap_or(false) || (label == "dock" && (shell_menu_open || crate::preview::is_open()));
                let probe = overlap_probe(label, r);
                let overlapped = others.iter().any(|o| rect_intersects(*o, probe));
                // Show the bars for a few seconds after launch so the user sees the app started.
                let grace = started.elapsed() < Duration::from_secs(4) && fs != Some(true);
                let held = label == "notch" && now_ms() < NOTCH_HOLD_UNTIL_MS.load(std::sync::atomic::Ordering::Relaxed);
                let want = !hidden_in_fullscreen(label, s.island.fullscreen, fs) && (grace || held || is_visible(Inputs { vis, fullscreen: fs, overlapped, cursor_near: near }));
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
    fn fixed_is_visible_unless_fullscreen() {
        assert!(is_visible(inp(Fixed, Some(false), true, false)));
        assert!(is_visible(inp(Fixed, None, true, false)));
        assert!(!is_visible(inp(Fixed, Some(true), false, false)));
    }

    #[test]
    fn fullscreen_reveals_every_mode_when_the_cursor_is_near() {
        for vis in [Fixed, Smart, Peek] {
            assert!(is_visible(inp(vis, Some(true), false, true)), "{vis:?} near");
            assert!(!is_visible(inp(vis, Some(true), false, false)), "{vis:?} away");
        }
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
    fn the_island_can_stay_out_of_fullscreen_apps_but_the_dock_cannot() {
        use FullscreenMode::{Edge, Hidden};
        assert!(hidden_in_fullscreen("notch", Hidden, Some(true)));
        assert!(!hidden_in_fullscreen("dock", Hidden, Some(true)), "the dock keeps its edge reveal");
        assert!(!hidden_in_fullscreen("notch", Edge, Some(true)), "edge is how it always worked");
        assert!(!hidden_in_fullscreen("notch", Hidden, Some(false)), "only in fullscreen");
        assert!(!hidden_in_fullscreen("notch", Hidden, None), "unknown counts as not fullscreen");
    }

    #[test]
    fn windows_own_menus_are_recognized_by_program_name() {
        assert!(is_shell_menu("StartMenuExperienceHost.exe"));
        assert!(is_shell_menu("SearchHost.exe"));
        assert!(is_shell_menu("shellexperiencehost.EXE"));
        assert!(!is_shell_menu("explorer.exe"));
        assert!(!is_shell_menu("macdows.exe"));
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
