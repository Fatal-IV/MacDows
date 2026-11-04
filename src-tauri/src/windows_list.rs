use base64::Engine;
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use windows::core::{PCWSTR, PWSTR};
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, WPARAM};
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
    /// AppUserModelID, "" when unknown; how a pinned Store app finds its windows.
    pub aumid: String,
    pub icon_png_base64: Option<String>,
}

/// Decides whether a window belongs in the dock.
pub fn is_listable(style: u32, ex_style: u32, cloaked: bool, title: &str) -> bool {
    style & WS_VISIBLE_BIT != 0 && ex_style & WS_EX_TOOLWINDOW_BIT == 0 && !cloaked && !title.trim().is_empty()
}

pub fn title_of(hwnd: HWND) -> String {
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

/// Handles of the notch and dock windows, which must never appear as dock icons.
static BARS: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());

pub fn is_bar_window(id: isize, bars: &[isize]) -> bool {
    bars.contains(&id)
}

/// Registers a window (notch or dock) as hidden from the dock list. Other MacDows windows,
/// such as Settings, are listed like any app.
pub fn register_bar(hwnd: isize) {
    let mut bars = BARS.lock().unwrap();
    if !bars.contains(&hwnd) {
        bars.push(hwnd);
    }
}

/// Store (MSIX) apps live under `WindowsApps`; the shell returns a generic icon for their exe
/// instead of failing, so the window's own icon is the better source.
pub fn prefer_window_icon(exe: &str) -> bool {
    exe.to_ascii_lowercase().contains(r"\windowsapps\")
}

pub fn icon_png_base64(exe: &str) -> Option<String> {
    if exe.is_empty() {
        return None;
    }
    let hicon = unsafe {
        let wide: Vec<u16> = exe.encode_utf16().chain(std::iter::once(0)).collect();
        let mut info = SHFILEINFOW::default();
        SHGetFileInfoW(
            PCWSTR(wide.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );
        info.hIcon
    };
    if hicon.is_invalid() {
        return None;
    }
    let result = hicon_to_png_base64(hicon);
    unsafe {
        let _ = DestroyIcon(hicon);
    }
    result
}

/// Icon the window itself provides. Needed for Store (MSIX) apps, whose exe icon cannot be read
/// from `Program Files\WindowsApps`. The handles are shared with the window, so they are not destroyed.
fn window_icon_png_base64(hwnd: HWND) -> Option<String> {
    let handle = unsafe {
        let mut found = 0usize;
        for kind in [1usize /* ICON_BIG */, 2 /* ICON_SMALL2 */] {
            let mut out = 0usize;
            let ok = SendMessageTimeoutW(hwnd, WM_GETICON, WPARAM(kind), LPARAM(0), SMTO_ABORTIFHUNG, 100, Some(&mut out));
            if ok.0 != 0 && out != 0 {
                found = out;
                break;
            }
        }
        if found == 0 {
            found = GetClassLongPtrW(hwnd, GCLP_HICON);
        }
        found
    };
    if handle == 0 {
        return None;
    }
    hicon_to_png_base64(HICON(handle as *mut _))
}

/// Converts an icon handle to a base64 PNG. Does not destroy the handle.
fn hicon_to_png_base64(hicon: HICON) -> Option<String> {
    unsafe {
        let mut ii = ICONINFO::default();
        let got = GetIconInfo(hicon, &mut ii).is_ok();
        let result = if got && !ii.hbmColor.is_invalid() { hbitmap_to_png_base64(ii.hbmColor) } else { None };
        if !ii.hbmColor.is_invalid() {
            let _ = DeleteObject(ii.hbmColor.into());
        }
        if !ii.hbmMask.is_invalid() {
            let _ = DeleteObject(ii.hbmMask.into());
        }
        result
    }
}

/// Encodes RGBA pixels (row by row, 4 bytes each) as a base64 PNG.
pub fn rgba_to_png_base64(w: u32, h: u32, px: &[u8]) -> Option<String> {
    let mut out = Vec::new();
    let mut written = false;
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        if let Ok(mut wr) = enc.write_header() {
            written = wr.write_image_data(px).is_ok() && wr.finish().is_ok();
        }
    }
    written.then(|| base64::engine::general_purpose::STANDARD.encode(&out))
}

/// Converts a bitmap to a base64 PNG (a bitmap without any alpha is made opaque). Does not delete the bitmap.
pub fn hbitmap_to_png_base64(bitmap: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<String> {
    unsafe {
        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        let dc = GetDC(None);
        let _ = GetDIBits(dc, bitmap, 0, 0, None, &mut bmi, DIB_RGB_COLORS);
        let (w, h) = (bmi.bmiHeader.biWidth, bmi.bmiHeader.biHeight.abs());
        let mut result = None;
        if w > 0 && h > 0 {
            let mut px = vec![0u8; (w * h * 4) as usize];
            bmi.bmiHeader.biHeight = -h;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB.0;
            let lines = GetDIBits(dc, bitmap, 0, h as u32, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
            if lines != 0 {
                let no_alpha = px.chunks(4).all(|p| p[3] == 0);
                for p in px.chunks_mut(4) {
                    p.swap(0, 2);
                    if no_alpha {
                        p[3] = 255;
                    }
                }
                result = rgba_to_png_base64(w as u32, h as u32, &px);
            }
        }
        ReleaseDC(None, dc);
        result
    }
}

/// The AppUserModelID of the app a window belongs to ("" when it has none): the package identity of
/// its process for Store apps, otherwise the property the window carries. Pinned Store apps match on it.
fn aumid_of(hwnd: HWND) -> String {
    use windows::Win32::Storage::Packaging::Appx::GetApplicationUserModelId;
    use windows::Win32::Foundation::PROPERTYKEY;
    use windows::Win32::System::Com::StructuredStorage::PropVariantToStringAlloc;
    use windows::Win32::UI::Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow};
    unsafe {
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            let mut buf = [0u16; 256];
            let mut len = buf.len() as u32;
            let status = GetApplicationUserModelId(process, &mut len, Some(PWSTR(buf.as_mut_ptr())));
            let _ = CloseHandle(process);
            if status.0 == 0 && len > 1 {
                return String::from_utf16_lossy(&buf[..len as usize - 1]);
            }
        }
        // System.AppUserModel.ID
        let key = PROPERTYKEY { fmtid: windows::core::GUID::from_u128(0x9F4C2855_9F79_4B39_A8D0_E1D42DE1D5F3), pid: 5 };
        let Ok(store) = SHGetPropertyStoreForWindow::<IPropertyStore>(hwnd) else { return String::new() };
        let Ok(value) = store.GetValue(&key) else { return String::new() };
        match PropVariantToStringAlloc(&value) {
            Ok(text) => {
                let out = text.to_string().unwrap_or_default();
                windows::Win32::System::Com::CoTaskMemFree(Some(text.0 as *const _));
                out
            }
            Err(_) => String::new(),
        }
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
    let (_, exe) = exe_of(hwnd);
    if is_bar_window(hwnd.0 as isize, &BARS.lock().unwrap()) {
        return BOOL(1);
    }
    let icon = if prefer_window_icon(&exe) {
        window_icon_png_base64(hwnd).or_else(|| icon_png_base64(&exe))
    } else {
        icon_png_base64(&exe).or_else(|| window_icon_png_base64(hwnd))
    };
    let aumid = aumid_of(hwnd);
    out.push(AppWindow { id: hwnd.0 as isize, title, exe, aumid, icon_png_base64: icon });
    BOOL(1)
}

unsafe extern "system" fn collect_rects(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<crate::fullscreen::Rect>);
    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
    let ex = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    let mut cloaked = 0u32;
    let _ = DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut cloaked as *mut _ as *mut _, 4);
    if !is_listable(style, ex, cloaked != 0, &title_of(hwnd)) || IsIconic(hwnd).as_bool() {
        return BOOL(1);
    }
    if matches!(class_of(hwnd).as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd") {
        return BOOL(1);
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == GetCurrentProcessId() {
        return BOOL(1);
    }
    let mut r = windows::Win32::Foundation::RECT::default();
    if GetWindowRect(hwnd, &mut r).is_ok() {
        out.push(crate::fullscreen::Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom });
    }
    BOOL(1)
}

/// Rectangles of visible, non-minimized app windows (no icon work), for overlap checks.
pub fn visible_rects() -> Vec<crate::fullscreen::Rect> {
    let mut out: Vec<crate::fullscreen::Rect> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect_rects), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

pub fn enumerate() -> Vec<AppWindow> {
    let mut out: Vec<AppWindow> = Vec::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut out as *mut _ as isize));
    }
    out
}

/// Keeps dock order stable: windows stay where they first appeared, new ones go to the end,
/// closed ones are forgotten. Windows reports windows in z-order, which would reshuffle the dock.
pub fn order_stable(known: &mut Vec<isize>, current: Vec<AppWindow>) -> Vec<AppWindow> {
    known.retain(|id| current.iter().any(|w| w.id == *id));
    for w in &current {
        if !known.contains(&w.id) {
            known.push(w.id);
        }
    }
    let mut out = current;
    out.sort_by_key(|w| known.iter().position(|id| *id == w.id));
    out
}

static ORDER: std::sync::Mutex<Vec<isize>> = std::sync::Mutex::new(Vec::new());

#[tauri::command]
pub fn list_windows() -> Vec<AppWindow> {
    order_stable(&mut ORDER.lock().unwrap(), enumerate())
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

    #[test]
    fn only_registered_bars_are_hidden_from_the_dock() {
        let bars = [11isize, 22];
        assert!(is_bar_window(11, &bars));
        assert!(is_bar_window(22, &bars));
        assert!(!is_bar_window(33, &bars));
        assert!(!is_bar_window(11, &[]));
    }

    #[test]
    fn store_apps_prefer_the_window_icon() {
        assert!(prefer_window_icon(r"C:\Program Files\WindowsApps\5319275A.WhatsAppDesktop_x64\WhatsApp.Root.exe"));
        assert!(prefer_window_icon(r"c:\program files\windowsapps\Foo\foo.exe"));
    }

    #[test]
    fn regular_apps_prefer_the_exe_icon() {
        assert!(!prefer_window_icon(r"C:\Program Files\Mozilla Firefox\firefox.exe"));
        assert!(!prefer_window_icon(""));
    }

    fn win(id: isize) -> AppWindow {
        AppWindow { id, title: format!("w{id}"), exe: String::new(), aumid: String::new(), icon_png_base64: None }
    }

    fn ids(v: &[AppWindow]) -> Vec<isize> {
        v.iter().map(|w| w.id).collect()
    }

    #[test]
    fn new_windows_are_appended_in_first_seen_order() {
        let mut known = Vec::new();
        let out = order_stable(&mut known, vec![win(3), win(1)]);
        assert_eq!(ids(&out), vec![3, 1]);
        let out = order_stable(&mut known, vec![win(3), win(1), win(2)]);
        assert_eq!(ids(&out), vec![3, 1, 2]);
    }

    #[test]
    fn z_order_changes_do_not_reorder() {
        let mut known = Vec::new();
        order_stable(&mut known, vec![win(1), win(2), win(3)]);
        let out = order_stable(&mut known, vec![win(3), win(1), win(2)]);
        assert_eq!(ids(&out), vec![1, 2, 3]);
    }

    #[test]
    fn closed_windows_are_forgotten_and_reopened_go_to_the_end() {
        let mut known = Vec::new();
        order_stable(&mut known, vec![win(1), win(2), win(3)]);
        let out = order_stable(&mut known, vec![win(1), win(3)]);
        assert_eq!(ids(&out), vec![1, 3]);
        let out = order_stable(&mut known, vec![win(2), win(1), win(3)]);
        assert_eq!(ids(&out), vec![1, 3, 2]);
    }
}
