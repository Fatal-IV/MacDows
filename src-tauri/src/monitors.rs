//! Monitors and the bars on them. This file starts with the pure part (which bar windows should exist for the monitors and the
//! settings); what creates, moves and removes the windows comes after it.

use crate::bar::{self, BarKind};
use crate::fullscreen::Rect;
use crate::settings::MonitorCfg;

/// One monitor as Windows reports it (physical pixels).
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub key: String,
    pub name: String,
    pub rect: Rect,
    /// The part of the monitor not taken by a taskbar.
    pub work: Rect,
    pub scale: f64,
    pub primary: bool,
}

/// What identifies a monitor in the settings: the hardware device ID (lowercased) because `\.\DISPLAYn` can be renumbered by Windows;
/// the display name only when no device ID can be read.
pub fn monitor_key(device_id: Option<&str>, name: &str) -> String {
    match device_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(id) => id.to_lowercase(),
        None => name.to_string(),
    }
}

/// Whether a monitor should have an island and a dock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Want {
    pub key: String,
    pub island: bool,
    pub dock: bool,
}

/// One entry per monitor that is plugged in: what the settings say, or (no entry yet) both bars on the primary monitor and none elsewhere.
pub fn desired(monitors: &[MonitorInfo], cfg: &[MonitorCfg]) -> Vec<Want> {
    monitors
        .iter()
        .map(|m| match cfg.iter().find(|c| c.key == m.key) {
            Some(c) => Want { key: m.key.clone(), island: c.island, dock: c.dock },
            None => Want { key: m.key.clone(), island: m.primary, dock: m.primary },
        })
        .collect()
}

pub fn is_wanted(want: &[Want], key: &str, kind: BarKind) -> bool {
    want.iter().find(|w| w.key == key).is_some_and(|w| match kind {
        BarKind::Notch => w.island,
        BarKind::Dock => w.dock,
    })
}

/// A bar window that exists, and the monitor it is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarWin {
    pub label: String,
    pub kind: BarKind,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Create { kind: BarKind, key: String },
    Destroy { label: String },
}

/// What to do so that the bar windows match the wishes. The plain `notch` and `dock` windows belong to the primary monitor, always exist
/// and are never in the result (a switched-off one is hidden by the visibility loop); numbered ones are created and destroyed.
/// Destroys come first.
pub fn plan(monitors: &[MonitorInfo], want: &[Want], existing: &[BarWin]) -> Vec<Action> {
    let mut actions = Vec::new();
    let numbered = |w: &&BarWin| matches!(bar::parse(&w.label), Some((_, Some(_))));
    for w in existing.iter().filter(numbered) {
        let monitor = monitors.iter().find(|m| m.key == w.key);
        let keep = monitor.is_some_and(|m| !m.primary) && is_wanted(want, &w.key, w.kind);
        if !keep {
            actions.push(Action::Destroy { label: w.label.clone() });
        }
    }
    for m in monitors.iter().filter(|m| !m.primary) {
        for kind in [BarKind::Notch, BarKind::Dock] {
            let has = existing.iter().filter(numbered).any(|w| w.key == m.key && w.kind == kind);
            if is_wanted(want, &m.key, kind) && !has {
                actions.push(Action::Create { kind, key: m.key.clone() });
            }
        }
    }
    actions
}

/// Changes whenever the layout does (monitors, positions, sizes, work areas, scales); the poller reconciles when it changes.
pub fn signature(monitors: &[MonitorInfo]) -> String {
    monitors
        .iter()
        .map(|m| {
            let (r, w) = (m.rect, m.work);
            format!("{}:{},{},{},{}:{},{},{},{}:{}:{}", m.key, r.left, r.top, r.right, r.bottom, w.left, w.top, w.right, w.bottom, m.scale, m.primary)
        })
        .collect::<Vec<_>>()
        .join("|")
}

// ---------------------------------------------------------------------------------------------
// Windows side: listing the monitors, creating, destroying and placing the bar windows.
// ---------------------------------------------------------------------------------------------
use crate::{windows_list, AppSettings};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

/// The logical size of a bar window, as in `tauri.conf.json`.
pub fn logical_size(kind: BarKind) -> (f64, f64) {
    match kind {
        BarKind::Notch => (580.0, 340.0),
        BarKind::Dock => (900.0, 120.0),
    }
}

/// Which monitor (key) each bar window is on. The plain `notch` and `dock` are on the primary monitor.
static BAR_MONITOR: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);
/// What the settings wanted at the last reconcile.
static WANT: Mutex<Vec<Want>> = Mutex::new(Vec::new());
static LAST_SIGNATURE: Mutex<String> = Mutex::new(String::new());
static NEXT_N: AtomicU32 = AtomicU32::new(2);
static RECONCILE: Mutex<()> = Mutex::new(());

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn monitor_of(label: &str) -> Option<String> {
    lock(&BAR_MONITOR).as_ref()?.get(label).cloned()
}

fn remember(label: &str, key: &str) {
    lock(&BAR_MONITOR).get_or_insert_with(HashMap::new).insert(label.to_string(), key.to_string());
}

fn forget(label: &str) {
    if let Some(map) = lock(&BAR_MONITOR).as_mut() {
        map.remove(label);
    }
}

/// The interface name of the monitor behind a display adapter (`\\.\DISPLAY1`): it stays the same when Windows renumbers the displays.
fn device_id(adapter: &str) -> Option<String> {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW};
    let wide: Vec<u16> = adapter.encode_utf16().chain(std::iter::once(0)).collect();
    let mut device = DISPLAY_DEVICEW { cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32, ..Default::default() };
    const EDD_GET_DEVICE_INTERFACE_NAME: u32 = 1;
    if !unsafe { EnumDisplayDevicesW(PCWSTR(wide.as_ptr()), 0, &mut device, EDD_GET_DEVICE_INTERFACE_NAME) }.as_bool() {
        return None;
    }
    let end = device.DeviceID.iter().position(|c| *c == 0).unwrap_or(device.DeviceID.len());
    let id = String::from_utf16_lossy(&device.DeviceID[..end]);
    (!id.is_empty()).then_some(id)
}

/// The monitors that are plugged in, in the order Windows reports them.
pub fn list(app: &AppHandle) -> Vec<MonitorInfo> {
    let primary = app.primary_monitor().ok().flatten().map(|m| *m.position());
    let Ok(monitors) = app.available_monitors() else { return Vec::new() };
    monitors
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let name = m.name().cloned().unwrap_or_default();
            let mut key = monitor_key(device_id(&name).as_deref(), &name);
            if key.is_empty() {
                key = format!("monitor-{i}");
            }
            let (p, s, w) = (m.position(), m.size(), m.work_area());
            MonitorInfo {
                key,
                name,
                rect: Rect { left: p.x, top: p.y, right: p.x + s.width as i32, bottom: p.y + s.height as i32 },
                work: Rect { left: w.position.x, top: w.position.y, right: w.position.x + w.size.width as i32, bottom: w.position.y + w.size.height as i32 },
                scale: m.scale_factor(),
                primary: primary == Some(*p),
            }
        })
        .collect()
}

/// Whether the settings want this bar window on its monitor. A window that is not a bar, or whose monitor is not known yet, counts as wanted.
pub fn is_enabled(label: &str) -> bool {
    let Some(kind) = bar::kind_of(label) else { return true };
    let Some(key) = monitor_of(label) else { return true };
    let want = lock(&WANT);
    want.is_empty() || is_wanted(&want, &key, kind)
}

/// Puts a bar window where it belongs on its monitor: the island centered on the top edge, the dock on the bottom of the work area.
/// A window of another monitor is sized for that monitor's scale first (its position first, so that it lands on that monitor).
fn place_with(app: &AppHandle, monitors: &[MonitorInfo], label: &str) {
    let Some((kind, number)) = bar::parse(label) else { return };
    let Some(win) = app.get_webview_window(label) else { return };
    let Some(monitor) = monitor_of(label).and_then(|key| monitors.iter().find(|m| m.key == key)) else { return };
    let before = win.outer_position().ok();
    if number.is_some() {
        let _ = win.set_position(PhysicalPosition::new(monitor.rect.left, monitor.rect.top));
        let (w, h) = logical_size(kind);
        let _ = win.set_size(PhysicalSize::new((w * monitor.scale).round() as u32, (h * monitor.scale).round() as u32));
    }
    let Ok(size) = win.outer_size() else { return };
    let (width, height) = (size.width as i32, size.height as i32);
    let (x, y) = match kind {
        BarKind::Notch => (monitor.rect.left + (monitor.rect.right - monitor.rect.left - width) / 2, monitor.rect.top),
        BarKind::Dock => (monitor.work.left + (monitor.work.right - monitor.work.left - width) / 2, monitor.work.bottom - height),
    };
    let _ = win.set_position(PhysicalPosition::new(x, y));
    // The shape the front end reported is in screen coordinates: it moves along with the window.
    if let (Some(from), Ok(to)) = (before, win.outer_position()) {
        if from != to {
            crate::hit_region::shift(label, to.x - from.x, to.y - from.y);
        }
    }
}

/// Puts the island and the dock of the primary monitor where they belong, at once (called while the app is starting, before anything
/// else: until then they sit at the default place of a new window and move around as the start goes on).
pub fn place_primary(app: &AppHandle) {
    let monitors = list(app);
    let Some(primary) = monitors.iter().find(|m| m.primary).or(monitors.first()) else { return };
    let key = primary.key.clone();
    for label in ["notch", "dock"] {
        remember(label, &key);
        place_with(app, &monitors, label);
    }
}

/// Places every bar window (after the work area changed, for example).
pub fn place_all(app: &AppHandle) {
    let monitors = list(app);
    for label in bar::bar_labels(app) {
        place_with(app, &monitors, &label);
    }
}

fn create(app: &AppHandle, kind: BarKind, key: &str) -> String {
    let label = bar::label(kind, Some(NEXT_N.fetch_add(1, Ordering::SeqCst)));
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = app.clone();
    let (l, k) = (label.clone(), key.to_string());
    let _ = app.run_on_main_thread(move || {
        let (w, h) = logical_size(kind);
        let title = if kind == BarKind::Notch { "MacDows Dynamic Island" } else { "MacDows Dock" };
        let built = WebviewWindowBuilder::new(&handle, &l, WebviewUrl::App("index.html".into()))
            .title(title)
            .inner_size(w, h)
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .shadow(false)
            .focused(false)
            .visible(false)
            .build();
        if let Ok(win) = built {
            if let Ok(hwnd) = win.hwnd() {
                windows_list::register_bar(hwnd.0 as isize);
            }
            remember(&l, &k);
        }
        let _ = tx.send(());
    });
    // Built on the main thread; wait for it (this runs on a worker thread) so that the next step finds the window.
    let _ = rx.recv_timeout(Duration::from_secs(5));
    label
}

fn destroy(app: &AppHandle, label: &str) {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = app.clone();
    let l = label.to_string();
    let _ = app.run_on_main_thread(move || {
        if let Some(win) = handle.get_webview_window(&l) {
            let _ = win.destroy();
        }
        let _ = tx.send(());
    });
    let _ = rx.recv_timeout(Duration::from_secs(5));
    forget(label);
    crate::hit_region::forget(label);
}

/// Makes the bar windows match the monitors and the settings: creates and destroys the numbered ones, places all of them.
/// Runs on a worker thread (never the main one: it waits for the main thread).
pub fn reconcile(app: &AppHandle) {
    let _guard = lock(&RECONCILE);
    let monitors = list(app);
    let Some(primary) = monitors.iter().find(|m| m.primary).or(monitors.first()) else { return };
    let primary_key = primary.key.clone();
    let cfg = app.state::<AppSettings>().0.lock().unwrap().monitors.clone();
    let want = desired(&monitors, &cfg);
    *lock(&WANT) = want.clone();
    remember("notch", &primary_key);
    remember("dock", &primary_key);
    // The bars of the primary monitor first: they exist from the start and must not wait for other monitors' windows to be built.
    for label in ["notch", "dock"] {
        place_with(app, &monitors, label);
    }
    let existing: Vec<BarWin> = bar::bar_labels(app)
        .into_iter()
        .filter_map(|label| {
            let (kind, _) = bar::parse(&label)?;
            Some(BarWin { kind, key: monitor_of(&label)?, label })
        })
        .collect();
    for action in plan(&monitors, &want, &existing) {
        match action {
            Action::Create { kind, key } => {
                // Placed right away: a new window is at a default place until then, and the visibility loop may show it any moment.
                let label = create(app, kind, &key);
                place_with(app, &monitors, &label);
            }
            Action::Destroy { label } => destroy(app, &label),
        }
    }
    place_all(app);
    *lock(&LAST_SIGNATURE) = signature(&monitors);
    sync_secondary_taskbars(app);
    let _ = app.emit("monitors-changed", ());
}

/// The label of the dock whose monitor contains the point (`docks`: each dock window with the rectangle of its monitor).
pub fn pick_dock(docks: &[(String, Rect)], point: (i32, i32)) -> Option<String> {
    docks.iter().find(|(_, r)| point.0 >= r.left && point.0 < r.right && point.1 >= r.top && point.1 < r.bottom).map(|(label, _)| label.clone())
}

/// The dock a keyboard shortcut belongs to: the one on the monitor under the pointer, else the primary dock.
pub fn dock_label_under_pointer(app: &AppHandle) -> String {
    let mut p = windows::Win32::Foundation::POINT::default();
    unsafe {
        let _ = windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p);
    }
    let monitors = list(app);
    let docks: Vec<(String, Rect)> = bar::bar_labels(app)
        .into_iter()
        .filter(|l| bar::kind_of(l) == Some(BarKind::Dock))
        .filter_map(|l| {
            let key = monitor_of(&l)?;
            let rect = monitors.iter().find(|m| m.key == key)?.rect;
            Some((l, rect))
        })
        .collect();
    pick_dock(&docks, (p.x, p.y)).unwrap_or_else(|| "dock".to_string())
}

/// The monitors that have a dock: their Windows taskbar is hidden while MacDows replaces it.
pub fn dock_rects(app: &AppHandle) -> Vec<Rect> {
    let want = lock(&WANT).clone();
    list(app).into_iter().filter(|m| is_wanted(&want, &m.key, BarKind::Dock)).map(|m| m.rect).collect()
}

/// Tells the taskbar code which monitors have a dock (nothing when MacDows does not replace the taskbar).
pub fn sync_secondary_taskbars(app: &AppHandle) {
    let replace = app.state::<AppSettings>().0.lock().unwrap().replace_taskbar;
    crate::taskbar::set_secondary(if replace { dock_rects(app) } else { Vec::new() });
}

/// Reconciles on a worker thread (for the settings command, which may run on the main thread).
pub fn reconcile_in_background(app: &AppHandle) {
    let app = app.clone();
    std::thread::spawn(move || reconcile(&app));
}

/// Places the first bars and then every two seconds looks whether monitors were plugged, unplugged, moved, rescaled or lost
/// or gained a taskbar; any change reconciles again.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        reconcile(&app);
        loop {
            std::thread::sleep(Duration::from_secs(2));
            let now = signature(&list(&app));
            let changed = *lock(&LAST_SIGNATURE) != now;
            if changed {
                reconcile(&app);
            }
        }
    });
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorView {
    pub key: String,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub x: i32,
    pub y: i32,
    pub primary: bool,
    pub island: bool,
    pub dock: bool,
}

/// The monitors that are plugged in with what the settings say about each (for the Monitors section of Settings).
#[tauri::command]
pub async fn get_monitors(app: AppHandle) -> Vec<MonitorView> {
    let monitors = list(&app);
    let cfg = app.state::<AppSettings>().0.lock().unwrap().monitors.clone();
    let want = desired(&monitors, &cfg);
    monitors
        .iter()
        .zip(want)
        .map(|(m, w)| MonitorView {
            key: m.key.clone(),
            name: m.name.clone(),
            width: m.rect.right - m.rect.left,
            height: m.rect.bottom - m.rect.top,
            x: m.rect.left,
            y: m.rect.top,
            primary: m.primary,
            island: w.island,
            dock: w.dock,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(left: i32, right: i32) -> Rect {
        Rect { left, top: 0, right, bottom: 1080 }
    }

    fn mon(key: &str, primary: bool, x: i32) -> MonitorInfo {
        MonitorInfo { key: key.into(), name: format!("name of {key}"), rect: r(x, x + 1920), work: r(x, x + 1920), scale: 1.0, primary }
    }

    fn cfg(key: &str, island: bool, dock: bool) -> MonitorCfg {
        MonitorCfg { key: key.into(), name: String::new(), island, dock }
    }

    fn want(key: &str, island: bool, dock: bool) -> Want {
        Want { key: key.into(), island, dock }
    }

    fn win(label: &str, key: &str) -> BarWin {
        let (kind, _) = bar::parse(label).unwrap();
        BarWin { label: label.into(), kind, key: key.into() }
    }

    #[test]
    fn the_key_prefers_the_device_id() {
        assert_eq!(monitor_key(Some("MONITOR\\DEL4012"), "\\\\.\\DISPLAY2"), "monitor\\del4012");
        assert_eq!(monitor_key(None, "\\\\.\\DISPLAY2"), "\\\\.\\DISPLAY2");
        assert_eq!(monitor_key(Some(""), "\\\\.\\DISPLAY2"), "\\\\.\\DISPLAY2");
        assert_eq!(monitor_key(Some("   "), "\\\\.\\DISPLAY2"), "\\\\.\\DISPLAY2");
    }

    #[test]
    fn by_default_only_the_primary_monitor_has_bars() {
        let w = desired(&[mon("a", true, 0), mon("b", false, 1920)], &[]);
        assert_eq!(w, vec![want("a", true, true), want("b", false, false)]);
    }

    #[test]
    fn a_saved_entry_wins() {
        let w = desired(&[mon("a", true, 0), mon("b", false, 1920)], &[cfg("a", false, true), cfg("b", true, true)]);
        assert_eq!(w, vec![want("a", false, true), want("b", true, true)]);
    }

    #[test]
    fn a_saved_monitor_that_is_not_plugged_in_is_ignored() {
        let w = desired(&[mon("a", true, 0)], &[cfg("gone", true, true)]);
        assert_eq!(w, vec![want("a", true, true)]);
    }

    #[test]
    fn a_wanted_second_monitor_gets_both_bars() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let actions = plan(&ms, &[want("a", true, true), want("b", true, true)], &[win("notch", "a"), win("dock", "a")]);
        assert_eq!(actions, vec![Action::Create { kind: BarKind::Notch, key: "b".into() }, Action::Create { kind: BarKind::Dock, key: "b".into() }]);
    }

    #[test]
    fn only_the_wanted_bar_is_created() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let actions = plan(&ms, &[want("a", true, true), want("b", false, true)], &[]);
        assert_eq!(actions, vec![Action::Create { kind: BarKind::Dock, key: "b".into() }]);
    }

    #[test]
    fn nothing_to_do_when_the_windows_exist() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let existing = [win("notch", "a"), win("dock", "a"), win("notch-2", "b"), win("dock-2", "b")];
        assert!(plan(&ms, &[want("a", true, true), want("b", true, true)], &existing).is_empty());
    }

    #[test]
    fn a_bar_that_is_switched_off_is_destroyed() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let existing = [win("notch-2", "b"), win("dock-2", "b")];
        let actions = plan(&ms, &[want("a", true, true), want("b", true, false)], &existing);
        assert_eq!(actions, vec![Action::Destroy { label: "dock-2".into() }]);
    }

    #[test]
    fn an_unplugged_monitor_destroys_only_its_windows() {
        let ms = [mon("a", true, 0), mon("c", false, 3840)];
        let existing = [win("dock-2", "b"), win("dock-3", "c")];
        let actions = plan(&ms, &[want("a", true, true), want("c", false, true)], &existing);
        assert_eq!(actions, vec![Action::Destroy { label: "dock-2".into() }]);
    }

    #[test]
    fn a_new_primary_monitor_loses_its_numbered_pair_and_the_old_primary_gets_one() {
        // b was a second monitor with notch-2 and dock-2; now b is the primary and a is the second one.
        let ms = [mon("a", false, 0), mon("b", true, 1920)];
        let existing = [win("notch", "a"), win("dock", "a"), win("notch-2", "b"), win("dock-2", "b")];
        let actions = plan(&ms, &[want("a", true, true), want("b", true, true)], &existing);
        assert_eq!(
            actions,
            vec![
                Action::Destroy { label: "notch-2".into() },
                Action::Destroy { label: "dock-2".into() },
                Action::Create { kind: BarKind::Notch, key: "a".into() },
                Action::Create { kind: BarKind::Dock, key: "a".into() },
            ]
        );
    }

    #[test]
    fn the_plain_windows_are_never_touched() {
        let ms = [mon("a", true, 0)];
        let actions = plan(&ms, &[want("a", false, false)], &[win("notch", "a"), win("dock", "a")]);
        assert!(actions.is_empty(), "a switched-off primary bar is hidden by the visibility loop, not closed");
    }

    #[test]
    fn is_wanted_reads_the_right_flag() {
        let w = [want("a", true, false)];
        assert!(is_wanted(&w, "a", BarKind::Notch));
        assert!(!is_wanted(&w, "a", BarKind::Dock));
        assert!(!is_wanted(&w, "missing", BarKind::Notch));
    }

    #[test]
    fn the_signature_changes_with_the_layout_only() {
        let a = [mon("a", true, 0), mon("b", false, 1920)];
        let same = [mon("a", true, 0), mon("b", false, 1920)];
        assert_eq!(signature(&a), signature(&same));
        let mut moved = same.clone();
        moved[1].rect = r(2400, 4320);
        assert_ne!(signature(&a), signature(&moved), "a monitor moved");
        let mut taskbar = same.clone();
        taskbar[0].work.bottom = 1040;
        assert_ne!(signature(&a), signature(&taskbar), "the work area changed");
        let mut scaled = same.clone();
        scaled[1].scale = 1.5;
        assert_ne!(signature(&a), signature(&scaled), "the scale changed");
        assert_ne!(signature(&a), signature(&a[..1]), "a monitor was unplugged");
    }

    #[test]
    fn the_shortcut_goes_to_the_dock_on_the_monitor_under_the_pointer() {
        let docks = [("dock".to_string(), r(0, 1920)), ("dock-2".to_string(), r(1920, 3840))];
        assert_eq!(pick_dock(&docks, (2000, 500)), Some("dock-2".to_string()));
        assert_eq!(pick_dock(&docks, (10, 10)), Some("dock".to_string()));
        assert_eq!(pick_dock(&docks, (1919, 1079)), Some("dock".to_string()), "the edges belong to the monitor on their inside");
        assert_eq!(pick_dock(&docks, (5000, 10)), None);
        assert_eq!(pick_dock(&[], (1, 1)), None);
    }
}
