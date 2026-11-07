//! Window previews: hovering a running app in the dock opens a small panel above it with one card per window
//! (a live thumbnail, the title and a close button). The panel is a window of its own (`preview`) that never
//! takes focus. It stays while the pointer is on the dock icon or on the panel, and closes shortly after.
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject, SetStretchBltMode, StretchBlt, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HALFTONE, SRCCOPY,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, IsIconic, PostMessageW, SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOWNOACTIVATE, WM_CLOSE};

const CARD_W: f64 = 200.0;
const CARD_H: f64 = 150.0;
const GAP: f64 = 10.0;
const PAD: f64 = 12.0;
const MAX_PER_ROW: usize = 5;
/// Thumbnails are drawn at most this big (CSS px of the card minus its title bar).
const THUMB_W: u32 = 200;
const THUMB_H: u32 = 112;
const CLOSE_DELAY_MS: u64 = 250;
const REFRESH_MS: u64 = 1000;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PreviewItem {
    pub hwnd: isize,
    pub title: String,
    /// A PNG of the window; `None` when it cannot be captured (minimized, drawn by the GPU) and the app icon is shown instead.
    pub thumbnail: Option<String>,
    /// The app's icon, the fallback picture.
    pub icon: Option<String>,
}

// ---------------------------------------------------------------------------------------------
// Pure parts
// ---------------------------------------------------------------------------------------------

/// The size that fits `max_w` x `max_h` with the same proportions; never larger than the original.
pub fn fit_size(w: u32, h: u32, max_w: u32, max_h: u32) -> (u32, u32) {
    if w == 0 || h == 0 {
        return (0, 0);
    }
    let scale = (max_w as f64 / w as f64).min(max_h as f64 / h as f64).min(1.0);
    (((w as f64 * scale).round() as u32).max(1), ((h as f64 * scale).round() as u32).max(1))
}

/// Close when neither the dock icon nor the panel has the pointer and that has lasted a moment (so moving
/// from the icon to the panel does not close it).
pub fn should_close(dock_hover: bool, panel_hover: bool, since_left_ms: u64) -> bool {
    !dock_hover && !panel_hover && since_left_ms >= CLOSE_DELAY_MS
}

/// Panel size in CSS px for `count` cards, and how many columns: `(width, height, columns)`.
pub fn layout(count: usize) -> (f64, f64, usize) {
    let count = count.max(1);
    let cols = count.min(MAX_PER_ROW);
    let rows = count.div_ceil(MAX_PER_ROW);
    (cols as f64 * CARD_W + (cols - 1) as f64 * GAP + 2.0 * PAD, rows as f64 * CARD_H + (rows - 1) as f64 * GAP + 2.0 * PAD, cols)
}

/// A capture that is entirely black (or fully transparent) is useless: some windows cannot be captured.
pub fn is_blank(rgba: &[u8]) -> bool {
    rgba.chunks_exact(4).all(|p| p[3] == 0 || (p[0] == 0 && p[1] == 0 && p[2] == 0))
}

// ---------------------------------------------------------------------------------------------
// Capture
// ---------------------------------------------------------------------------------------------

/// A thumbnail of the window as a base64 PNG, or `None` (minimized, gone, empty or black).
pub fn thumbnail(hwnd: isize, max_w: u32, max_h: u32) -> Option<String> {
    unsafe {
        let window = HWND(hwnd as *mut _);
        if IsIconic(window).as_bool() {
            return None;
        }
        let mut rect = RECT::default();
        GetWindowRect(window, &mut rect).ok()?;
        let (w, h) = ((rect.right - rect.left).max(0) as u32, (rect.bottom - rect.top).max(0) as u32);
        let (tw, th) = fit_size(w, h, max_w, max_h);
        if tw == 0 || th == 0 {
            return None;
        }
        let screen = GetDC(None);
        let full_dc = CreateCompatibleDC(Some(screen));
        let full = CreateCompatibleBitmap(screen, w as i32, h as i32);
        let full_old = SelectObject(full_dc, full.into());
        // PW_RENDERFULLCONTENT (2) also captures windows that are drawn through DirectComposition.
        let printed = PrintWindow(window, full_dc, PRINT_WINDOW_FLAGS(2)).as_bool();

        let small_dc = CreateCompatibleDC(Some(screen));
        let small = CreateCompatibleBitmap(screen, tw as i32, th as i32);
        let small_old = SelectObject(small_dc, small.into());
        SetStretchBltMode(small_dc, HALFTONE);
        let scaled = printed && StretchBlt(small_dc, 0, 0, tw as i32, th as i32, Some(full_dc), 0, 0, w as i32, h as i32, SRCCOPY).as_bool();
        // A bitmap can only be read while it is not selected into a device context.
        SelectObject(small_dc, small_old);
        SelectObject(full_dc, full_old);

        let mut result = None;
        if scaled {
            let mut bmi = BITMAPINFO::default();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = tw as i32;
            bmi.bmiHeader.biHeight = -(th as i32);
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB.0;
            let mut px = vec![0u8; (tw * th * 4) as usize];
            let lines = GetDIBits(screen, small, 0, th, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
            if lines != 0 {
                for p in px.chunks_exact_mut(4) {
                    p.swap(0, 2); // BGRA to RGBA
                    p[3] = 255;
                }
                if !is_blank(&px) {
                    result = crate::windows_list::rgba_to_png_base64(tw, th, &px);
                }
            }
        }
        let _ = DeleteObject(small.into());
        let _ = DeleteObject(full.into());
        let _ = DeleteDC(small_dc);
        let _ = DeleteDC(full_dc);
        ReleaseDC(None, screen);
        result
    }
}

// ---------------------------------------------------------------------------------------------
// The panel
// ---------------------------------------------------------------------------------------------

static ITEMS: Mutex<Vec<PreviewItem>> = Mutex::new(Vec::new());
static DOCK_HOVER: AtomicBool = AtomicBool::new(false);
static PANEL_HOVER: AtomicBool = AtomicBool::new(false);
/// Milliseconds (since `now_ms`'s start) when neither had the pointer; 0 while one has it.
static LEFT_AT: AtomicU64 = AtomicU64::new(0);
/// True while a panel is open (or about to open); the dock stays in view meanwhile, or moving the pointer from the
/// icon up to the panel would hide the dock and take the panel with it.
static PANEL_OPEN: AtomicBool = AtomicBool::new(false);

pub fn is_open() -> bool {
    PANEL_OPEN.load(Ordering::SeqCst)
}

/// Clears `PANEL_OPEN` when the monitor of its panel ends, unless a newer panel has taken over.
struct OpenGuard(u64);
impl Drop for OpenGuard {
    fn drop(&mut self) {
        if GENERATION.load(Ordering::SeqCst) == self.0 {
            PANEL_OPEN.store(false, Ordering::SeqCst);
        }
    }
}

/// Which panel (by a counter) the monitor thread belongs to; a newer panel retires the older monitor.
static GENERATION: AtomicU64 = AtomicU64::new(0);

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(1).max(1)
}

fn refresh_items(hwnds: &[isize], icon: &Option<String>) -> Vec<PreviewItem> {
    hwnds
        .iter()
        .map(|&hwnd| PreviewItem { hwnd, title: crate::windows_list::title_of(HWND(hwnd as *mut _)), thumbnail: thumbnail(hwnd, THUMB_W, THUMB_H), icon: icon.clone() })
        .collect()
}

/// Opens (or updates) the panel above the dock icon. `center_x` and `dock_top` are CSS px in the dock window.
#[tauri::command]
pub fn show_preview(app: AppHandle, window: tauri::WebviewWindow, center_x: f64, dock_top: f64, icon: Option<String>, hwnds: Vec<isize>) {
    if hwnds.is_empty() {
        return;
    }
    DOCK_HOVER.store(true, Ordering::SeqCst);
    LEFT_AT.store(0, Ordering::SeqCst);
    PANEL_OPEN.store(true, Ordering::SeqCst);
    // The panel opens above the dock that asked (there is one dock per monitor).
    let dock_label = window.label().to_string();
    std::thread::spawn(move || {
        let items = refresh_items(&hwnds, &icon);
        *ITEMS.lock().unwrap() = items;
        let Some(dock) = app.get_webview_window(&dock_label) else { return };
        let (Ok(pos), Ok(Some(monitor))) = (dock.outer_position(), dock.current_monitor()) else {
            PANEL_OPEN.store(false, Ordering::SeqCst);
            return;
        };
        let scale = dock.scale_factor().unwrap_or(1.0);
        let (w, h, _) = layout(hwnds.len());
        let work = monitor.work_area();
        let x = (pos.x as f64 + center_x * scale - w * scale / 2.0).clamp(work.position.x as f64 + 8.0, (work.position.x + work.size.width as i32) as f64 - w * scale - 8.0);
        let y = pos.y as f64 + dock_top * scale - h * scale - 10.0 * scale;
        let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            if let Some(existing) = handle.get_webview_window("preview") {
                let _ = existing.set_size(tauri::LogicalSize::new(w, h));
                let _ = existing.set_position(PhysicalPosition::new(x as i32, y as i32));
                let _ = handle.emit_to("preview", "preview-updated", ());
                return;
            }
            let built = WebviewWindowBuilder::new(&handle, "preview", WebviewUrl::App("index.html".into()))
                .title("MacDows preview")
                .inner_size(w, h)
                .decorations(false)
                .transparent(true)
                .resizable(false)
                .shadow(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .visible(false)
                .build();
            if let Ok(win) = built {
                if let Ok(hwnd) = win.hwnd() {
                    crate::windows_list::register_bar(hwnd.0 as isize);
                    let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
                    unsafe {
                        let _ = ShowWindow(HWND(hwnd.0), SW_SHOWNOACTIVATE);
                    }
                }
            }
        });
        monitor_panel(app, generation, hwnds, icon, dock_label);
    });
}

/// Keeps the thumbnails fresh and closes the panel when the pointer has left (or the dock hid).
fn monitor_panel(app: AppHandle, generation: u64, hwnds: Vec<isize>, icon: Option<String>, dock_label: String) {
    let _guard = OpenGuard(generation);
    let mut last_refresh = now_ms();
    loop {
        std::thread::sleep(std::time::Duration::from_millis(80));
        if GENERATION.load(Ordering::SeqCst) != generation {
            return; // a newer panel took over
        }
        let Some(panel) = app.get_webview_window("preview") else {
            // The panel may not exist yet (it is built on the main thread) or is already gone.
            if now_ms().saturating_sub(last_refresh) > 3000 {
                return;
            }
            continue;
        };
        let dock_visible = app.get_webview_window(&dock_label).and_then(|d| d.is_visible().ok()).unwrap_or(false);
        let left = LEFT_AT.load(Ordering::SeqCst);
        let since = if left == 0 { 0 } else { now_ms().saturating_sub(left) };
        let hovering = DOCK_HOVER.load(Ordering::SeqCst) || PANEL_HOVER.load(Ordering::SeqCst);
        if !hovering && left == 0 {
            LEFT_AT.store(now_ms(), Ordering::SeqCst);
        } else if hovering && left != 0 {
            LEFT_AT.store(0, Ordering::SeqCst);
        }
        if !dock_visible || should_close(DOCK_HOVER.load(Ordering::SeqCst), PANEL_HOVER.load(Ordering::SeqCst), since) {
            let _ = panel.destroy();
            return;
        }
        if now_ms().saturating_sub(last_refresh) >= REFRESH_MS {
            last_refresh = now_ms();
            let still: Vec<isize> = hwnds.iter().copied().filter(|h| ITEMS.lock().unwrap().iter().any(|i| i.hwnd == *h)).collect();
            if still.is_empty() {
                let _ = panel.destroy();
                return;
            }
            *ITEMS.lock().unwrap() = refresh_items(&still, &icon);
            let _ = app.emit_to("preview", "preview-updated", ());
        }
    }
}

/// The pointer entered or left the dock icon (`source` "dock") or the panel ("panel").
#[tauri::command]
pub fn preview_hover(source: String, inside: bool) {
    match source.as_str() {
        "dock" => DOCK_HOVER.store(inside, Ordering::SeqCst),
        "panel" => PANEL_HOVER.store(inside, Ordering::SeqCst),
        _ => {}
    }
}

#[tauri::command]
pub fn preview_items() -> Vec<PreviewItem> {
    ITEMS.lock().unwrap().clone()
}

fn close_panel(app: &AppHandle) {
    GENERATION.fetch_add(1, Ordering::SeqCst);
    PANEL_OPEN.store(false, Ordering::SeqCst);
    DOCK_HOVER.store(false, Ordering::SeqCst);
    PANEL_HOVER.store(false, Ordering::SeqCst);
    if let Some(w) = app.get_webview_window("preview") {
        let _ = w.destroy();
    }
}

#[tauri::command]
pub fn close_preview(app: AppHandle) {
    close_panel(&app);
}

/// Brings the window forward (restoring it if it was minimized) and closes the panel.
#[tauri::command]
pub fn preview_activate(app: AppHandle, hwnd: isize) {
    unsafe {
        let window = HWND(hwnd as *mut _);
        if IsIconic(window).as_bool() {
            let _ = ShowWindow(window, SW_RESTORE);
        }
        let _ = SetForegroundWindow(window);
    }
    close_panel(&app);
}

/// Asks the window to close and takes its card away; an empty panel closes.
#[tauri::command]
pub fn preview_close_window(app: AppHandle, hwnd: isize) {
    unsafe {
        let _ = PostMessageW(Some(HWND(hwnd as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0));
    }
    let empty = {
        let mut items = ITEMS.lock().unwrap();
        items.retain(|i| i.hwnd != hwnd);
        items.is_empty()
    };
    if empty {
        close_panel(&app);
    } else {
        let _ = app.emit_to("preview", "preview-updated", ());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_big_window_is_scaled_down_keeping_its_proportions() {
        assert_eq!(fit_size(1920, 1080, 200, 120), (200, 113));
        assert_eq!(fit_size(1000, 2000, 200, 120), (60, 120));
    }

    #[test]
    fn a_small_window_is_never_scaled_up() {
        assert_eq!(fit_size(100, 50, 200, 120), (100, 50));
    }

    #[test]
    fn an_empty_window_gives_no_size() {
        assert_eq!(fit_size(0, 0, 200, 120), (0, 0));
        assert_eq!(fit_size(10, 0, 200, 120), (0, 0));
    }

    #[test]
    fn the_panel_closes_only_after_the_pointer_has_been_away_a_moment() {
        assert!(should_close(false, false, 250));
        assert!(!should_close(false, false, 100), "too soon: the pointer may be on its way to the panel");
        assert!(!should_close(true, false, 1000));
        assert!(!should_close(false, true, 1000));
        assert!(!should_close(true, true, 1000));
    }

    #[test]
    fn the_panel_grows_by_cards_and_wraps_after_five() {
        let (w1, h1, c1) = layout(1);
        assert_eq!((c1, w1, h1), (1, 200.0 + 24.0, 150.0 + 24.0));
        let (w5, _, c5) = layout(5);
        assert_eq!(c5, 5);
        assert_eq!(w5, 5.0 * 200.0 + 4.0 * 10.0 + 24.0);
        let (w6, h6, c6) = layout(6);
        assert_eq!(c6, 5);
        assert_eq!(w6, w5);
        assert_eq!(h6, 2.0 * 150.0 + 10.0 + 24.0);
        assert_eq!(layout(0), layout(1), "never an empty panel");
    }

    #[test]
    fn black_or_transparent_captures_are_blank() {
        assert!(is_blank(&[0, 0, 0, 255, 0, 0, 0, 255]));
        assert!(is_blank(&[10, 20, 30, 0, 0, 0, 0, 0]));
        assert!(!is_blank(&[0, 0, 0, 255, 0, 1, 0, 255]));
        assert!(is_blank(&[]), "nothing captured is blank");
    }

    #[test]
    fn this_pc_can_capture_a_real_window() {
        // Any visible top-level window of the desktop, e.g. the shell's own: just must not crash or hang.
        let desktop = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow() };
        let _ = thumbnail(desktop.0 as isize, THUMB_W, THUMB_H);
    }
}
