use base64::Engine;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use windows::core::{PCWSTR, PWSTR};
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS};
use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
use windows::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
use windows::Win32::UI::WindowsAndMessaging::*;

const WS_VISIBLE_BIT: u32 = 0x1000_0000;
const WS_EX_TOOLWINDOW_BIT: u32 = 0x0000_0080;

#[derive(Debug, Clone, Serialize)]
pub struct AppWindow {
    pub id: isize,
    pub title: String,
    pub exe: String,
    pub icon_png_base64: Option<String>,
}

/// Decides whether a window belongs in the dock.
pub fn is_listable(style: u32, ex_style: u32, cloaked: bool, title: &str) -> bool {
    style & WS_VISIBLE_BIT != 0 && ex_style & WS_EX_TOOLWINDOW_BIT == 0 && !cloaked && !title.trim().is_empty()
}

fn title_of(hwnd: HWND) -> String {
    let len = unsafe { GetWindowTextLengthW(hwnd) };
    if len <= 0 {
        return String::new();
    }
    let mut buf = vec![0u16; len as usize + 1];
    let n = unsafe { GetWindowTextW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

fn class_of(hwnd: HWND) -> String {
    let mut buf = [0u16; 64];
    let n = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    String::from_utf16_lossy(&buf[..n])
}

fn exe_of(hwnd: HWND) -> (u32, String) {
    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let Ok(h) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return (pid, String::new());
    };
    let mut buf = [0u16; 520];
    let mut len = buf.len() as u32;
    let ok = unsafe { QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len) }.is_ok();
    let _ = unsafe { CloseHandle(h) };
    (pid, if ok { String::from_utf16_lossy(&buf[..len as usize]) } else { String::new() })
}

fn icon_png_base64(exe: &str) -> Option<String> {
    if exe.is_empty() {
        return None;
    }
    unsafe {
        let wide: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();
        let mut info = SHFILEINFOW::default();
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        if info.hIcon.is_invalid() {
            return None;
        }
        let mut ii = ICONINFO::default();
        let got = GetIconInfo(info.hIcon, &mut ii).is_ok();
        let mut result = None;
        if got && !ii.hbmColor.is_invalid() {
            let mut bmi = BITMAPINFO::default();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            let dc = GetDC(None);
            let _ = GetDIBits(dc, ii.hbmColor, 0, 0, None, &mut bmi, DIB_RGB_COLORS);
            let (w, h) = (bmi.bmiHeader.biWidth, bmi.bmiHeader.biHeight.abs());
            if w > 0 && h > 0 {
                let mut px = vec![0u8; (w * h * 4) as usize];
                bmi.bmiHeader.biHeight = -h;
                bmi.bmiHeader.biBitCount = 32;
                bmi.bmiHeader.biCompression = BI_RGB.0;
                let lines = GetDIBits(dc, ii.hbmColor, 0, h as u32, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
                if lines != 0 {
                    let no_alpha = px.chunks(4).all(|p| p[3] == 0);
                    for p in px.chunks_mut(4) {
                        p.swap(0, 2);
                        if no_alpha {
                            p[3] = 255;
                        }
                    }
                    let mut out = Vec::new();
                    let mut written = false;
                    {
                        let mut enc = png::Encoder::new(&mut out, w as u32, h as u32);
                        enc.set_color(png::ColorType::Rgba);
                        enc.set_depth(png::BitDepth::Eight);
                        if let Ok(mut wr) = enc.write_header() {
                            written = wr.write_image_data(&px).is_ok() && wr.finish().is_ok();
                        }
                    }
                    if written {
                        result = Some(base64::engine::general_purpose::STANDARD.encode(&out));
                    }
                }
            }
            ReleaseDC(None, dc);
        }
        if !ii.hbmColor.is_invalid() {
            let _ = DeleteObject(ii.hbmColor.into());
        }
        if !ii.hbmMask.is_invalid() {
            let _ = DeleteObject(ii.hbmMask.into());
        }
        let _ = DestroyIcon(info.hIcon);
        result
    }
}

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<AppWindow>);
    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
    let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    let mut cloaked = 0u32;
    let _ = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4);
    let title = title_of(hwnd);
    if !is_listable(style, ex, cloaked != 0, &title) {
        return BOOL(1);
    }
    if matches!(class_of(hwnd).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd") {
        return BOOL(1);
    }
    let (pid, exe) = exe_of(hwnd);
    if pid == GetCurrentProcessId() {
        return BOOL(1);
    }
    let icon = icon_png_base64(&exe);
    out.push(AppWindow { id: hwnd.0 as isize, title, exe, icon_png_base64: icon });
    BOOL(1)
}

pub fn enumerate() -> Vec<AppWindow> {
    let mut out: Vec<AppWindow> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

#[tauri::command]
pub fn list_windows() -> Vec<AppWindow> {
    enumerate()
}

#[tauri::command]
pub fn focus_window(id: isize) {
    unsafe {
        let hwnd = HWND(id as *mut _);
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
    }
}

static APP: OnceLock<AppHandle> = OnceLock::new();
static PENDING: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn on_event(_: HWINEVENTHOOK, _: u32, _: HWND, id_object: i32, _: i32, _: u32, _: u32) {
    if id_object != OBJID_WINDOW.0 || PENDING.swap(true, Ordering::SeqCst) {
        return;
    }
    // Debounce bursts of events into one `windows-changed`.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(120));
        PENDING.store(false, Ordering::SeqCst);
        if let Some(app) = APP.get() {
            let _ = app.emit("windows-changed", ());
        }
    });
}

const EVENT_OBJECT_CLOAKED: u32 = 0x8017;
const EVENT_OBJECT_UNCLOAKED: u32 = 0x8018;

/// Event-driven: create/destroy/show/hide/rename/cloak. No polling.
pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    std::thread::spawn(|| unsafe {
        for (lo, hi) in [
            (EVENT_OBJECT_CREATE, EVENT_OBJECT_HIDE),
            (EVENT_OBJECT_NAMECHANGE, EVENT_OBJECT_NAMECHANGE),
            (EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED),
        ] {
            let _ = SetWinEventHook(lo, hi, None, Some(on_event), 0, 0, WINEVENT_OUTOFCONTEXT);
        }
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

    const VISIBLE: u32 = 0x1000_0000;
    const TOOLWINDOW: u32 = 0x0000_0080;

    #[test]
    fn normal_titled_window_is_listed() {
        assert!(is_listable(VISIBLE, 0, false, "Notepad"));
    }

    #[test]
    fn empty_title_is_rejected() {
        assert!(!is_listable(VISIBLE, 0, false, ""));
        assert!(!is_listable(VISIBLE, 0, false, "   "));
    }

    #[test]
    fn tool_window_is_rejected() {
        assert!(!is_listable(VISIBLE, TOOLWINDOW, false, "Palette"));
    }

    #[test]
    fn cloaked_window_is_rejected() {
        assert!(!is_listable(VISIBLE, 0, true, "Settings"));
    }

    #[test]
    fn invisible_window_is_rejected() {
        assert!(!is_listable(0, 0, false, "Hidden"));
    }
}
