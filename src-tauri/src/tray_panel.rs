//! The "hidden icons" panel, drawn by MacDows instead of Windows.
//!
//! Windows owns the notification-area icons, so the only way to reach them is its own overflow flyout.
//! We open that flyout invisibly (fully transparent), read its icons and names with UI Automation, copy
//! each icon with `PrintWindow`, and hide it again. MacDows then shows the icons in its own themed window
//! above the dock, and a click is forwarded to the real icon through UI Automation.

use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{COLORREF, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationInvokePattern, TreeScope_Descendants, UIA_InvokePatternId};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowLongW, GetWindowRect, IsWindowVisible, SetForegroundWindow,
    SetLayeredWindowAttributes, SetWindowLongW, ShowWindow, GWL_EXSTYLE, LWA_ALPHA, SW_HIDE, SW_SHOWNA, WS_EX_LAYERED, WS_EX_TRANSPARENT,
};
use windows::core::{BOOL, Interface};

const OVERFLOW_CLASS: &str = "TopLevelWindowForOverflowXamlIsland";
const ITEM_ID: &str = "NotifyItemIcon";

#[derive(Serialize, Clone, Debug)]
pub struct TrayItem {
    pub index: usize,
    pub name: String,
    pub icon_png_base64: Option<String>,
}

static ITEMS: Mutex<Vec<TrayItem>> = Mutex::new(Vec::new());
/// When the list above was read last.
static COLLECTED_AT: Mutex<Option<Instant>> = Mutex::new(None);

/// A list read this recently is used as it is when the panel is opened.
const FRESH: Duration = Duration::from_secs(60);
/// Hovering the chevron reads the list again unless it was read this recently.
const PREFETCH_AFTER: Duration = Duration::from_secs(5);

/// Where the items of a click on the chevron come from. Reading them takes about 0.7 s (the system's own flyout has to be opened out of
/// sight, read and closed), so it is done when the pointer reaches the chevron, and the click normally finds the result waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickSource {
    /// Open the panel with what was read before.
    Cache,
    /// A look is running and there is nothing to show yet: wait for it.
    Wait,
    /// Read the list now.
    Collect,
}

pub fn click_source(cache_empty: bool, age: Option<Duration>, look_running: bool) -> ClickSource {
    let fresh = age.is_some_and(|a| a < FRESH);
    if !cache_empty && (fresh || look_running) {
        ClickSource::Cache
    } else if cache_empty && look_running {
        ClickSource::Wait
    } else {
        ClickSource::Collect
    }
}

pub fn should_prefetch(look_running: bool, age: Option<Duration>) -> bool {
    !look_running && age.map_or(true, |a| a >= PREFETCH_AFTER)
}
static BUSY: AtomicBool = AtomicBool::new(false);

/// True while the flyout is being read, so the taskbar watchdog does not hide the taskbar underneath it.
pub fn is_busy() -> bool {
    BUSY.load(Ordering::SeqCst)
}

struct Busy;
impl Busy {
    fn try_new() -> Option<Self> {
        (!BUSY.swap(true, Ordering::SeqCst)).then_some(Busy)
    }
}
impl Drop for Busy {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::SeqCst);
    }
}

/// Why the hidden-icons panel did not open, for `%APPDATA%\MacDows\logs	ray.log`.
fn trace(msg: &str) {
    use std::io::Write;
    let dir = std::env::var_os("APPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir).join("MacDows").join("logs");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("tray.log")) {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = writeln!(f, "[{secs}] {msg}");
    }
}

fn class_name(hwnd: HWND) -> String {
    let mut buf = [0u16; 128];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) }.max(0) as usize;
    String::from_utf16_lossy(&buf[..len])
}

/// First top-level window of a class. `FindWindow` misses some of these, so the windows are enumerated.
fn find_by_class(class: &'static str, only_visible: bool) -> Option<HWND> {
    struct Search {
        found: Option<HWND>,
        class: &'static str,
        only_visible: bool,
    }
    unsafe extern "system" fn each(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let search = unsafe { &mut *(lparam.0 as *mut Search) };
        if class_name(hwnd) == search.class && (!search.only_visible || unsafe { IsWindowVisible(hwnd) }.as_bool()) {
            search.found = Some(hwnd);
            return BOOL(0);
        }
        BOOL(1)
    }
    let mut search = Search { found: None, class, only_visible };
    unsafe {
        let _ = EnumWindows(Some(each), LPARAM(&mut search as *mut Search as isize));
    }
    search.found
}

/// The overflow flyout window. It keeps existing (hidden) after the first time it was opened.
fn find_overflow(only_visible: bool) -> Option<HWND> {
    find_by_class(OVERFLOW_CLASS, only_visible)
}

/// While the (hidden) taskbar is briefly shown, fully transparent and click-through, so UI Automation can see its
/// buttons. Puts it back as it was when dropped.
struct TrayShown {
    hwnd: HWND,
    original_ex: i32,
    was_hidden: bool,
}

impl TrayShown {
    fn new() -> Option<Self> {
        let hwnd = find_by_class("Shell_TrayWnd", false)?;
        let was_hidden = !unsafe { IsWindowVisible(hwnd) }.as_bool();
        let original_ex = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
        if was_hidden {
            unsafe {
                SetWindowLongW(hwnd, GWL_EXSTYLE, original_ex | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32);
                let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
                let _ = ShowWindow(hwnd, SW_SHOWNA);
            }
            std::thread::sleep(Duration::from_millis(150));
        }
        Some(TrayShown { hwnd, original_ex, was_hidden })
    }
}

impl Drop for TrayShown {
    fn drop(&mut self) {
        if self.was_hidden {
            unsafe {
                let _ = ShowWindow(self.hwnd, SW_HIDE);
                SetWindowLongW(self.hwnd, GWL_EXSTYLE, self.original_ex);
            }
        }
    }
}

/// Whether a taskbar item (its automation id and its name) is the "show hidden icons" chevron.
fn is_chevron_item(id: &str, name: &str) -> bool {
    let name = name.to_lowercase();
    id == "NotificationChevron" || id == "OverflowButton" || name.contains("hidden icons") || name.contains("gizli simge")
}

/// One look at the taskbar's UI Automation tree: the chevron was pressed (or could not be), or it is not there (yet); then the number of
/// elements and the named ones are returned for the log.
enum Look {
    Pressed(bool),
    Missing { count: i32, seen: Vec<String> },
}

fn look_for_chevron(automation: &IUIAutomation, tray: HWND) -> Option<Look> {
    unsafe {
        let root = automation.ElementFromHandle(tray).ok()?;
        let condition = automation.CreateTrueCondition().ok()?;
        let all = root.FindAll(TreeScope_Descendants, &condition).ok()?;
        let count = all.Length().unwrap_or(0);
        let mut seen: Vec<String> = Vec::new();
        for i in 0..count {
            let Ok(el) = all.GetElement(i) else { continue };
            let id = el.CurrentAutomationId().map(|s| s.to_string()).unwrap_or_default();
            let name = el.CurrentName().map(|s| s.to_string()).unwrap_or_default();
            if seen.len() < 60 && (!id.is_empty() || !name.is_empty()) {
                let class = el.CurrentClassName().map(|s| s.to_string()).unwrap_or_default();
                seen.push(format!("{}|{id}|{class}", name.to_lowercase()));
            }
            if !is_chevron_item(&id, &name) {
                continue;
            }
            if let Ok(pattern) = el.GetCurrentPattern(UIA_InvokePatternId)
                && let Ok(invoke) = pattern.cast::<IUIAutomationInvokePattern>()
            {
                return Some(Look::Pressed(invoke.Invoke().is_ok()));
            }
        }
        Some(Look::Missing { count, seen })
    }
}

/// Presses the taskbar's "show hidden icons" chevron through UI Automation, looking again for a short while (the tray may need a moment).
/// False when the chevron is not in the tree or could not be pressed (newer Windows 11 builds do not offer the tray to UI Automation
/// at all: only the task list shows up); the log then says what the taskbar did offer.
/// How long the chevron is looked for through UI Automation before the keyboard is used instead.
const CHEVRON_WAIT: Duration = Duration::from_millis(300);

/// Set once the chevron had to be pressed with the keyboard: this Windows build does not offer the tray to UI Automation, so the next
/// time the keyboard is used right away (no time lost looking). Cleared when the keyboard fails, so that a later Windows update that
/// fixes the tree is noticed.
static UIA_LACKS_TRAY: AtomicBool = AtomicBool::new(false);

fn press_chevron(automation: &IUIAutomation, tray: HWND) -> bool {
    let started = Instant::now();
    loop {
        match look_for_chevron(automation, tray) {
            Some(Look::Pressed(ok)) => {
                if started.elapsed() > Duration::from_millis(400) {
                    trace(&format!("the chevron appeared after {} ms", started.elapsed().as_millis()));
                }
                return ok;
            }
            Some(Look::Missing { count, seen }) => {
                if started.elapsed() >= CHEVRON_WAIT {
                    trace(&format!("{count} elements in the taskbar, named ones: {}", seen.join(" ; ")));
                    return false;
                }
            }
            None => {
                if started.elapsed() >= CHEVRON_WAIT {
                    trace("the taskbar could not be read through UI Automation");
                    return false;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(120));
    }
}

/// Keeps the flyout fully transparent while it is open, and puts its window style back afterwards.
struct Invisible {
    hwnd: HWND,
    original_ex: i32,
}

impl Invisible {
    fn apply(hwnd: HWND) -> Self {
        let original_ex = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
        unsafe {
            SetWindowLongW(hwnd, GWL_EXSTYLE, original_ex | WS_EX_LAYERED.0 as i32);
            let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA);
        }
        Invisible { hwnd, original_ex }
    }
}

impl Drop for Invisible {
    fn drop(&mut self) {
        unsafe {
            let _ = SetLayeredWindowAttributes(self.hwnd, COLORREF(0), 255, LWA_ALPHA);
            SetWindowLongW(self.hwnd, GWL_EXSTYLE, self.original_ex);
        }
    }
}

/// Opens the flyout by pressing the taskbar's chevron and makes it invisible as early as possible. `None` when
/// there is nothing hidden (no chevron) or the flyout did not appear.
fn open_flyout(automation: &IUIAutomation) -> Option<(HWND, Invisible)> {
    // The window survives its first use: make it transparent before it is shown so it never flashes.
    let mut early = find_overflow(false).map(Invisible::apply);
    {
        let Some(tray) = TrayShown::new() else {
            trace("the taskbar window (Shell_TrayWnd) was not found");
            return None;
        };
        let by_uia = !UIA_LACKS_TRAY.load(Ordering::SeqCst) && press_chevron(automation, tray.hwnd);
        if !by_uia {
            // Win+B, Home, Enter: the keyboard reaches the chevron even where UI Automation does not show the tray.
            if !UIA_LACKS_TRAY.load(Ordering::SeqCst) {
                trace("the chevron is not in the UI Automation tree: opening the hidden icons with the keyboard");
            }
            crate::taskbar::run_steps(&crate::taskbar_plan::key_steps(crate::taskbar_plan::ShellAction::HiddenIcons));
        }
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(3) {
            if let Some(hwnd) = find_overflow(true) {
                UIA_LACKS_TRAY.store(!by_uia, Ordering::SeqCst);
                let guard = early.take().unwrap_or_else(|| Invisible::apply(hwnd));
                // The XAML content needs a moment to build after the window appears.
                std::thread::sleep(Duration::from_millis(350));
                return Some((hwnd, guard));
            }
            std::thread::sleep(Duration::from_millis(8));
        }
    }
    trace("the overflow flyout did not appear within 3 seconds");
    UIA_LACKS_TRAY.store(false, Ordering::SeqCst);
    None
}

/// Closes the flyout. Esc only when it has the focus, so it never reaches another window.
fn close_flyout(hwnd: HWND, previous: HWND) {
    unsafe {
        if GetForegroundWindow() == hwnd {
            crate::taskbar::run_steps(&[crate::taskbar_plan::KeyStep::Chord { win: false, key: 0x1B }]);
            std::thread::sleep(Duration::from_millis(120));
        }
        if IsWindowVisible(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
        if !previous.0.is_null() {
            let _ = SetForegroundWindow(previous);
        }
    }
}

struct RawItem {
    name: String,
    icon: RECT,
    button: RECT,
}

fn read_items(automation: &IUIAutomation, hwnd: HWND) -> Vec<RawItem> {
    let mut items = Vec::new();
    unsafe {
        let Ok(root) = automation.ElementFromHandle(hwnd) else { return items };
        let Ok(condition) = automation.CreateTrueCondition() else { return items };
        let Ok(all) = root.FindAll(TreeScope_Descendants, &condition) else { return items };
        let count = all.Length().unwrap_or(0);
        let mut i = 0;
        while i < count {
            if let Ok(el) = all.GetElement(i) {
                let id = el.CurrentAutomationId().map(|s| s.to_string()).unwrap_or_default();
                if id == ITEM_ID {
                    let name = el.CurrentName().map(|s| s.to_string()).unwrap_or_default().trim().to_string();
                    let button = el.CurrentBoundingRectangle().unwrap_or_default();
                    // The icon image is the next element in document order.
                    let icon = all
                        .GetElement(i + 1)
                        .ok()
                        .and_then(|e| e.CurrentBoundingRectangle().ok())
                        .filter(|r| r.right > r.left)
                        .unwrap_or(button);
                    items.push(RawItem { name, icon, button });
                }
            }
            i += 1;
        }
    }
    items
}

/// BGRA, top-down copy of the window as DWM composes it (works while the window is fully transparent).
fn capture(hwnd: HWND) -> Option<(Vec<u8>, i32, i32, i32, i32)> {
    unsafe {
        let mut r = RECT::default();
        GetWindowRect(hwnd, &mut r).ok()?;
        let (w, h) = (r.right - r.left, r.bottom - r.top);
        if w <= 0 || h <= 0 {
            return None;
        }
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        let dib = CreateDIBSection(Some(mem), &bmi, DIB_RGB_COLORS, &mut bits, None, 0);
        let result = match dib {
            Ok(dib) if !bits.is_null() => {
                let old = SelectObject(mem, dib.into());
                // PW_RENDERFULLCONTENT (2) also captures XAML/DirectComposition content.
                let ok = PrintWindow(hwnd, mem, PRINT_WINDOW_FLAGS(2)).as_bool();
                let data = ok.then(|| std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize).to_vec());
                SelectObject(mem, old);
                let _ = DeleteObject(dib.into());
                data.map(|d| (d, w, h, r.left, r.top))
            }
            _ => None,
        };
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        result
    }
}

/// Cuts one icon out of the captured flyout and turns the flyout's flat background into transparency.
fn icon_png(frame: &(Vec<u8>, i32, i32, i32, i32), icon: RECT) -> Option<String> {
    let (data, w, h, left, top) = frame;
    // A little margin around the 20px icon, so nothing is cut at the edge.
    let pad = ((icon.right - icon.left) as f32 * 0.2).round() as i32;
    let (x0, y0) = ((icon.left - left - pad).max(0), (icon.top - top - pad).max(0));
    let (x1, y1) = ((icon.right - left + pad).min(*w), (icon.bottom - top + pad).min(*h));
    let (cw, ch) = (x1 - x0, y1 - y0);
    if cw < 4 || ch < 4 {
        return None;
    }
    let px = |x: i32, y: i32| -> [u8; 3] {
        let i = ((y * w + x) * 4) as usize;
        [data[i + 2], data[i + 1], data[i]]
    };
    // The flyout's own background: the median of the crop's outer ring.
    let mut ring: Vec<[u8; 3]> = Vec::new();
    for x in x0..x1 {
        ring.push(px(x, y0));
        ring.push(px(x, y1 - 1));
    }
    for y in y0..y1 {
        ring.push(px(x0, y));
        ring.push(px(x1 - 1, y));
    }
    let median = |c: usize| {
        let mut v: Vec<u8> = ring.iter().map(|p| p[c]).collect();
        v.sort_unstable();
        v[v.len() / 2] as f32
    };
    let bg = [median(0), median(1), median(2)];

    let mut rgba = Vec::with_capacity((cw * ch * 4) as usize);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = px(x, y);
            let d = ((p[0] as f32 - bg[0]).powi(2) + (p[1] as f32 - bg[1]).powi(2) + (p[2] as f32 - bg[2]).powi(2)).sqrt();
            let a = ((d - 6.0) / 40.0).clamp(0.0, 1.0);
            if a == 0.0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                // Undo the blend with the background so edges keep the icon's own color.
                let c = |i: usize| (bg[i] + (p[i] as f32 - bg[i]) / a).clamp(0.0, 255.0) as u8;
                rgba.extend_from_slice(&[c(0), c(1), c(2), (a * 255.0) as u8]);
            }
        }
    }
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, cw as u32, ch as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(&rgba).ok()?;
    }
    use base64::Engine;
    Some(base64::engine::general_purpose::STANDARD.encode(out))
}

fn automation() -> Option<IUIAutomation> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()
    }
}

/// Opens the Windows flyout invisibly, reads every icon, closes it again. `None` when it could not be opened.
pub fn collect() -> Option<Vec<TrayItem>> {
    let _busy = Busy::try_new()?;
    let started = Instant::now();
    let automation = automation()?;
    let previous = unsafe { GetForegroundWindow() };
    let (hwnd, guard) = open_flyout(&automation)?;
    let raw = read_items(&automation, hwnd);
    if raw.is_empty() {
        trace("the flyout opened but no icons were found in it");
    }
    let frame = capture(hwnd);
    // Close first, then make it opaque again: the other way round leaves it visible for a moment.
    close_flyout(hwnd, previous);
    drop(guard);
    let items: Vec<TrayItem> = raw
        .iter()
        .enumerate()
        .map(|(index, r)| TrayItem { index, name: r.name.clone(), icon_png_base64: frame.as_ref().and_then(|f| icon_png(f, r.icon)) })
        .collect();
    *ITEMS.lock().unwrap() = items.clone();
    *COLLECTED_AT.lock().unwrap() = Some(Instant::now());
    trace(&format!("{} hidden icons read in {} ms", items.len(), started.elapsed().as_millis()));
    Some(items)
}

/// Clicks the icon at `index` (as listed by the last `collect`). The name must still match.
pub fn activate(index: usize, expected_name: &str) -> bool {
    let Some(_busy) = Busy::try_new() else { return false };
    let Some(automation) = automation() else { return false };
    let previous = unsafe { GetForegroundWindow() };
    let Some((hwnd, guard)) = open_flyout(&automation) else { return false };
    let raw = read_items(&automation, hwnd);
    let mut invoked = false;
    if let Some(item) = raw.get(index).filter(|r| r.name == expected_name) {
        let _ = item;
        unsafe {
            if let Ok(root) = automation.ElementFromHandle(hwnd)
                && let Ok(condition) = automation.CreateTrueCondition()
                && let Ok(all) = root.FindAll(TreeScope_Descendants, &condition)
            {
                let mut seen = 0usize;
                for i in 0..all.Length().unwrap_or(0) {
                    let Ok(el) = all.GetElement(i) else { continue };
                    if el.CurrentAutomationId().map(|s| s.to_string()).unwrap_or_default() != ITEM_ID {
                        continue;
                    }
                    if seen == index {
                        if let Ok(pattern) = el.GetCurrentPattern(UIA_InvokePatternId)
                            && let Ok(invoke) = pattern.cast::<IUIAutomationInvokePattern>()
                        {
                            invoked = invoke.Invoke().is_ok();
                        }
                        break;
                    }
                    seen += 1;
                }
            }
        }
    }
    close_flyout(hwnd, if invoked { HWND(std::ptr::null_mut()) } else { previous });
    drop(guard);
    invoked
}

pub fn last_items() -> Vec<TrayItem> {
    ITEMS.lock().unwrap().clone()
}

fn cache_age() -> Option<Duration> {
    COLLECTED_AT.lock().unwrap().map(|at| at.elapsed())
}

/// The items for a click on the chevron: the stored list at once when there is a usable one, else (after a running look) a new reading.
pub fn items_for_click() -> Option<Vec<TrayItem>> {
    let cached = last_items();
    match click_source(cached.is_empty(), cache_age(), is_busy()) {
        ClickSource::Cache => Some(cached),
        ClickSource::Wait => {
            let started = Instant::now();
            while is_busy() && started.elapsed() < Duration::from_secs(3) {
                std::thread::sleep(Duration::from_millis(20));
            }
            Some(last_items()).filter(|i| !i.is_empty()).or_else(collect)
        }
        ClickSource::Collect => collect(),
    }
}

/// Reads the hidden icons ahead of the click (called when the pointer reaches the chevron, and once after start).
pub fn prefetch() {
    if should_prefetch(is_busy(), cache_age()) {
        let _ = collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live check against the real taskbar: `cargo test --lib live_collect -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn live_dump_tray() {
        let automation = automation().expect("uia");
        let tray = TrayShown::new().expect("tray");
        std::thread::sleep(Duration::from_millis(2000));
        unsafe {
            let mut r = RECT::default();
            let _ = GetWindowRect(tray.hwnd, &mut r);
            println!("tray visible={} rect=({},{},{},{}) was_hidden={}", IsWindowVisible(tray.hwnd).as_bool(), r.left, r.top, r.right, r.bottom, tray.was_hidden);
            let root = automation.ElementFromHandle(tray.hwnd).expect("root");
            let all = root.FindAll(TreeScope_Descendants, &automation.CreateTrueCondition().unwrap()).expect("all");
            println!("elements: {}", all.Length().unwrap_or(0));
            for i in 0..all.Length().unwrap_or(0) {
                let el = all.GetElement(i).unwrap();
                println!("{} | {} | {}", el.CurrentAutomationId().map(|s| s.to_string()).unwrap_or_default(), el.CurrentName().map(|s| s.to_string()).unwrap_or_default(), el.CurrentClassName().map(|s| s.to_string()).unwrap_or_default());
            }
        }
    }

    #[test]
    #[ignore]
    fn live_collect() {
        let items = collect();
        println!("collect -> {:?}", items.as_ref().map(|v| v.iter().map(|i| i.name.clone()).collect::<Vec<_>>()));
    }

    #[test]
    fn the_background_becomes_transparent_and_the_icon_keeps_its_color() {
        // A 20x20 flyout area with a flat grey background and a red 6x6 square in the middle.
        let (w, h) = (20, 20);
        let mut data = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let i = ((y * w + x) * 4) as usize;
                let red = (7..13).contains(&x) && (7..13).contains(&y);
                let (r, g, b) = if red { (220, 30, 30) } else { (44, 44, 44) };
                data[i] = b;
                data[i + 1] = g;
                data[i + 2] = r;
                data[i + 3] = 255;
            }
        }
        let frame = (data, w, h, 0, 0);
        let png_b64 = icon_png(&frame, RECT { left: 5, top: 5, right: 15, bottom: 15 }).expect("png");
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD.decode(png_b64).unwrap();
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();
        let (cw, ch) = (info.width as usize, info.height as usize);
        let at = |x: usize, y: usize| &buf[(y * cw + x) * 4..(y * cw + x) * 4 + 4];
        assert_eq!(at(0, 0)[3], 0, "background corner is transparent");
        let centre = at(cw / 2, ch / 2);
        assert_eq!(centre[3], 255, "icon pixel is opaque");
        assert!(centre[0] > 200 && centre[1] < 60, "icon keeps its red color, got {centre:?}");
    }

    #[test]
    fn the_chevron_is_recognised_by_id_or_by_name() {
        assert!(is_chevron_item("NotificationChevron", ""));
        assert!(is_chevron_item("OverflowButton", ""));
        assert!(is_chevron_item("", "Show Hidden Icons"));
        assert!(is_chevron_item("", "Gizli simgeleri göster"));
        assert!(!is_chevron_item("", "çalışan uygulamalar"));
        assert!(!is_chevron_item("Start", "start"));
    }

    #[test]
    fn a_fresh_cache_opens_the_panel_at_once() {
        assert_eq!(click_source(false, Some(Duration::from_secs(10)), false), ClickSource::Cache);
        assert_eq!(click_source(false, Some(Duration::from_secs(59)), false), ClickSource::Cache);
    }

    #[test]
    fn a_stale_cache_is_still_used_while_a_look_is_running() {
        assert_eq!(click_source(false, Some(Duration::from_secs(600)), true), ClickSource::Cache);
    }

    #[test]
    fn a_stale_cache_with_no_look_running_is_read_again() {
        assert_eq!(click_source(false, Some(Duration::from_secs(61)), false), ClickSource::Collect);
        assert_eq!(click_source(false, None, false), ClickSource::Collect);
    }

    #[test]
    fn with_nothing_cached_the_click_waits_for_a_running_look_or_looks_itself() {
        assert_eq!(click_source(true, None, true), ClickSource::Wait);
        assert_eq!(click_source(true, None, false), ClickSource::Collect);
        assert_eq!(click_source(true, Some(Duration::from_secs(1)), false), ClickSource::Collect, "an empty list is no cache");
    }

    #[test]
    fn hovering_looks_again_only_when_the_last_look_is_a_few_seconds_old() {
        assert!(should_prefetch(false, None));
        assert!(should_prefetch(false, Some(Duration::from_secs(6))));
        assert!(!should_prefetch(false, Some(Duration::from_secs(2))));
        assert!(!should_prefetch(true, None), "one look at a time");
    }
}
