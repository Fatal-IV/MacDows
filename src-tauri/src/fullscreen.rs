use crate::settings::Visibility;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetClassNameW, GetForegroundWindow, GetMessageW, GetWindowRect, TranslateMessage, EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

pub fn covers_monitor(win: Rect, monitor: Rect) -> bool {
    win.left <= monitor.left && win.top <= monitor.top && win.right >= monitor.right && win.bottom >= monitor.bottom
}

/// A known fullscreen app hides every mode. When the state is unknown, Fixed stays visible
/// while Smart and Peek hide.
pub fn should_hide(vis: Visibility, fullscreen: Option<bool>) -> bool {
    match vis {
        Visibility::Fixed => fullscreen == Some(true),
        _ => fullscreen.unwrap_or(true),
    }
}

static APP: OnceLock<AppHandle> = OnceLock::new();
static LAST: std::sync::atomic::AtomicI8 = std::sync::atomic::AtomicI8::new(-1);

fn class_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

fn probe() -> Option<bool> {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return None;
        }
        if matches!(class_of(fg).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd") {
            return Some(false);
        }
        let mut wr = RECT::default();
        GetWindowRect(fg, &mut wr).ok()?;
        let mon = MonitorFromWindow(fg, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(mon, &mut mi).as_bool() {
            return None;
        }
        let m = mi.rcMonitor;
        Some(covers_monitor(
            Rect { left: wr.left, top: wr.top, right: wr.right, bottom: wr.bottom },
            Rect { left: m.left, top: m.top, right: m.right, bottom: m.bottom },
        ))
    }
}

/// Current state: Some(true/false) when known, None before the first probe or if probing failed.
pub fn state() -> Option<bool> {
    match LAST.load(std::sync::atomic::Ordering::Relaxed) {
        1 => Some(true),
        0 => Some(false),
        _ => None,
    }
}

unsafe extern "system" fn on_event(_: HWINEVENTHOOK, _: u32, _: HWND, id_object: i32, _: i32, _: u32, _: u32) {
    if id_object != 0 {
        return; // only OBJID_WINDOW
    }
    update();
}

fn update() {
    let state = probe();
    let code = match state { Some(true) => 1, Some(false) => 0, None => 2 };
    if LAST.swap(code, std::sync::atomic::Ordering::Relaxed) == code {
        return;
    }
    if let Some(app) = APP.get() {
        let _ = app.emit("fullscreen-changed", state.unwrap_or(true));
    }
}

/// Starts an event-driven watcher (no polling): foreground change and foreground-window move/resize.
pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    update();
    std::thread::spawn(|| unsafe {
        let _ = SetWinEventHook(EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, None, Some(on_event), 0, 0, WINEVENT_OUTOFCONTEXT);
        let _ = SetWinEventHook(EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_LOCATIONCHANGE, None, Some(on_event), 0, 0, WINEVENT_OUTOFCONTEXT);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Visibility;

    const MON: Rect = Rect { left: 0, top: 0, right: 1920, bottom: 1080 };

    #[test]
    fn exact_cover_is_fullscreen() {
        assert!(covers_monitor(MON, MON));
    }

    #[test]
    fn smaller_window_is_not() {
        let win = Rect { left: 0, top: 0, right: 1920, bottom: 1040 };
        assert!(!covers_monitor(win, MON));
    }

    #[test]
    fn fixed_mode_hides_only_for_known_fullscreen() {
        assert!(should_hide(Visibility::Fixed, Some(true)));
        assert!(!should_hide(Visibility::Fixed, Some(false)));
        assert!(!should_hide(Visibility::Fixed, None));
    }

    #[test]
    fn unknown_state_hides_in_smart_and_peek() {
        assert!(should_hide(Visibility::Smart, None));
        assert!(should_hide(Visibility::Peek, None));
    }

    #[test]
    fn known_state_decides_in_smart() {
        assert!(should_hide(Visibility::Smart, Some(true)));
        assert!(!should_hide(Visibility::Smart, Some(false)));
    }
}
