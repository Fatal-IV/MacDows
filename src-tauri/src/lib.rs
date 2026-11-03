mod i18n;
mod cli;
mod taskbar;
mod taskbar_plan;
mod keyhook;
mod launcher;
mod search;
mod tray_panel;
mod updates;
mod volume;
mod visibility;
mod wallpaper;
mod media;
mod windows_list;
mod appearance;
mod fullscreen;
mod settings;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindowBuilder};

pub struct AppSettings(Mutex<settings::Settings>);

#[tauri::command]
fn get_settings(state: State<AppSettings>) -> settings::Settings {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn set_setting(app: AppHandle, state: State<AppSettings>, key: String, value: serde_json::Value) -> Result<settings::Settings, String> {
    let mut guard = state.0.lock().unwrap();
    let mut json = serde_json::to_value(&*guard).map_err(|e| e.to_string())?;
    json.as_object_mut().ok_or("settings not an object")?.insert(key.clone(), value);
    let updated: settings::Settings = serde_json::from_value(json).map_err(|e| e.to_string())?;
    settings::save(&updated).map_err(|e| e.to_string())?;
    *guard = updated.clone();
    drop(guard);
    let _ = app.emit("settings-changed", updated.clone());
    match key.as_str() {
        "replace_taskbar" => apply_replace_taskbar(&app, updated.replace_taskbar),
        "start_with_windows" => taskbar::set_autostart(updated.start_with_windows),
        _ => {}
    }
    Ok(updated)
}

/// Hides or restores the Windows taskbar off the UI thread, then moves the dock to the new work area.
fn apply_replace_taskbar(app: &AppHandle, enabled: bool) {
    let app = app.clone();
    std::thread::spawn(move || {
        if enabled {
            if taskbar::hide_now() {
                taskbar::start_guard();
                // With Windows's taskbar gone, the Windows key and Win+S belong to MacDows's menus.
                keyhook::set_enabled(true);
            }
        } else {
            keyhook::set_enabled(false);
            taskbar::restore_now();
        }
        // The work area changes a moment after the taskbar does.
        std::thread::sleep(std::time::Duration::from_millis(1500));
        place_dock(&app);
        let _ = app.emit("taskbar-changed", ());
    });
}

/// Turns the setting off from outside the Settings window (tray menu) and restores the taskbar.
fn disable_replace_taskbar(app: &AppHandle) {
    let state = app.state::<AppSettings>();
    let updated = {
        let mut guard = state.0.lock().unwrap();
        guard.replace_taskbar = false;
        let _ = settings::save(&guard);
        guard.clone()
    };
    let _ = app.emit("settings-changed", updated);
    apply_replace_taskbar(app, false);
}

/// First-run notice (once), then hide. Also hides again whenever Explorer restarts.
fn start_taskbar_takeover(app: &AppHandle) {
    use taskbar_plan::Store;
    let (replace, notice_shown, lang, autostart) = {
        let s = app.state::<AppSettings>();
        let g = s.0.lock().unwrap();
        (g.replace_taskbar, g.taskbar_notice_shown, g.language.clone(), g.start_with_windows)
    };
    taskbar::set_autostart(autostart);
    // A crash can leave the taskbar hidden when even the guard was killed: put it back before anything else.
    let state = taskbar::FileStore::default().load();
    let alive = state.as_ref().map_or(false, |s| taskbar::pid_alive(s.app_pid));
    if taskbar_plan::needs_stale_restore(state.as_ref(), alive) {
        taskbar::restore_now();
    }
    if !taskbar_plan::should_replace_now(replace, cfg!(debug_assertions), std::env::var_os("MACDOWS_REPLACE_TASKBAR").is_some()) {
        return;
    }
    let handle = app.clone();
    std::thread::spawn(move || {
        if !notice_shown {
            let (title, text) = i18n::taskbar_notice(&lang);
            taskbar::show_notice(title, text);
            let s = handle.state::<AppSettings>();
            let mut g = s.0.lock().unwrap();
            g.taskbar_notice_shown = true;
            let _ = settings::save(&g);
        }
        apply_replace_taskbar(&handle, true);
    });
    let handle = app.clone();
    taskbar::watch_taskbar_created(move || {
        let still_on = handle.state::<AppSettings>().0.lock().unwrap().replace_taskbar;
        if still_on {
            taskbar::hide_now();
            std::thread::sleep(std::time::Duration::from_millis(1500));
            place_dock(&handle);
            let _ = handle.emit("taskbar-changed", ());
        }
    });
    let handle = app.clone();
    taskbar::start_watchdog(move || handle.state::<AppSettings>().0.lock().unwrap().replace_taskbar);
}


/// When each popup last closed because it lost focus, so the click that did it does not reopen it at once.
static POPUP_CLOSED_AT_MS: Mutex<Vec<(String, u64)>> = Mutex::new(Vec::new());

fn note_popup_closed(label: &str) {
    let mut list = POPUP_CLOSED_AT_MS.lock().unwrap();
    list.retain(|(l, _)| l != label);
    list.push((label.to_string(), now_ms()));
}

fn popup_recently_closed(label: &str) -> bool {
    POPUP_CLOSED_AT_MS.lock().unwrap().iter().any(|(l, at)| l == label && now_ms().saturating_sub(*at) < 500)
}

#[tauri::command]
fn close_popup(app: AppHandle, label: String) {
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.close();
        note_popup_closed(&label);
    }
}

/// Opens a themed popup (`label` is also the view the frontend shows) centered above the dock, or closes it when
/// it is already open. `dock_top` is the dock's top edge in CSS pixels of the dock window.
fn toggle_popup(app: &AppHandle, label: &'static str, title: &'static str, size: (f64, f64), dock_top: f64) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.close();
        return;
    }
    if popup_recently_closed(label) {
        return;
    }
    let Some(dock) = app.get_webview_window("dock") else { return };
    let (Ok(pos), Ok(Some(monitor))) = (dock.outer_position(), dock.current_monitor()) else { return };
    let scale = dock.scale_factor().unwrap_or(1.0);
    let (work_x, work_y, work_w) = {
        let work = monitor.work_area();
        (work.position.x as f64, work.position.y as f64, work.size.width as f64)
    };
    let (w, h) = size;
    let x = work_x + (work_w - w * scale) / 2.0;
    let y = pos.y as f64 + dock_top * scale - h * scale - 12.0 * scale;
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        let built = WebviewWindowBuilder::new(&handle, label, WebviewUrl::App("index.html".into()))
            .title(title)
            .inner_size(w, h)
            .decorations(false)
            .transparent(true)
            .resizable(false)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .focused(true)
            .visible(false)
            .build();
        match built {
            Ok(win) => {
                if let Ok(hwnd) = win.hwnd() {
                    windows_list::register_bar(hwnd.0 as isize);
                }
                let moved = win.set_position(PhysicalPosition::new(x as i32, y.max(work_y) as i32));
                let shown = win.show();
                let focused = win.set_focus();
                eprintln!("popup {label}: move={moved:?} show={shown:?} focus={focused:?} at ({x:.0},{y:.0})");
            }
            Err(e) => eprintln!("popup {label}: could not be created: {e}"),
        }
    });
}

#[tauri::command]
fn toggle_start_menu(app: AppHandle, dock_top: f64) {
    eprintln!("popup start: toggle requested, dock_top={dock_top}");
    // Windows are created from a worker thread, as for the hidden-icons panel: creating one inside a synchronous
    // command (which runs on the main thread) leaves it half-built.
    std::thread::spawn(move || toggle_popup(&app, "start", "MacDows Start", (640.0, 700.0), dock_top));
}

const TRAY_CELL: f64 = 46.0;
const TRAY_PAD: f64 = 10.0;
const TRAY_COLUMNS: usize = 5;

/// When the panel last closed because it lost focus. A click on the dock button that did it must not reopen it.
static TRAY_CLOSED_AT_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// Closes the hidden-icons panel when it is open. Returns whether it was.
fn close_tray(app: &AppHandle) -> bool {
    match app.get_webview_window("tray") {
        Some(w) => {
            let _ = w.close();
            true
        }
        None => false,
    }
}

#[tauri::command]
fn close_tray_panel(app: AppHandle) {
    if close_tray(&app) {
        TRAY_CLOSED_AT_MS.store(now_ms(), std::sync::atomic::Ordering::SeqCst);
    }
}

#[tauri::command]
fn get_tray_items() -> Vec<tray_panel::TrayItem> {
    tray_panel::last_items()
}

/// Clicks a hidden icon, then closes the panel.
#[tauri::command]
fn activate_tray_item(app: AppHandle, index: usize, name: String) {
    close_tray(&app);
    std::thread::spawn(move || {
        tray_panel::activate(index, &name);
    });
}

#[tauri::command]
fn toggle_search_menu(app: AppHandle, dock_top: f64) {
    std::thread::spawn(move || toggle_popup(&app, "search", "MacDows Search", (640.0, 560.0), dock_top));
}

/// The dock tells where its top edge is, so the menus opened from the keyboard sit right above it.
#[tauri::command]
fn report_dock_top(top: f64) {
    keyhook::set_dock_top(top);
}

/// Called from the keyboard hook's worker thread.
pub(crate) fn open_start_from_key(app: &AppHandle, dock_top: f64) {
    toggle_popup(app, "start", "MacDows Start", (640.0, 700.0), dock_top);
}

pub(crate) fn open_search_from_key(app: &AppHandle, dock_top: f64) {
    toggle_popup(app, "search", "MacDows Search", (640.0, 560.0), dock_top);
}

/// Opens (or closes) the hidden-icons panel just above the dock. `center_x` and `dock_top` are CSS pixels in
/// the dock window.
#[tauri::command]
fn toggle_tray_panel(app: AppHandle, center_x: f64, dock_top: f64) {
    if close_tray(&app) || now_ms().saturating_sub(TRAY_CLOSED_AT_MS.load(std::sync::atomic::Ordering::SeqCst)) < 500 {
        return;
    }
    std::thread::spawn(move || {
        let Some(items) = tray_panel::collect().filter(|i| !i.is_empty()) else { return };
        let Some(dock) = app.get_webview_window("dock") else { return };
        let (Ok(pos), Ok(Some(monitor))) = (dock.outer_position(), dock.current_monitor()) else { return };
        let scale = dock.scale_factor().unwrap_or(1.0);
        let columns = items.len().min(TRAY_COLUMNS);
        let rows = items.len().div_ceil(TRAY_COLUMNS);
        let (w, h) = (columns as f64 * TRAY_CELL + TRAY_PAD * 2.0, rows as f64 * TRAY_CELL + TRAY_PAD * 2.0);
        let work = monitor.work_area();
        let x = (pos.x as f64 + center_x * scale - w * scale / 2.0)
            .clamp(work.position.x as f64 + 8.0, (work.position.x + work.size.width as i32) as f64 - w * scale - 8.0);
        let y = pos.y as f64 + dock_top * scale - h * scale - 10.0 * scale;
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            let built = WebviewWindowBuilder::new(&handle, "tray", WebviewUrl::App("index.html".into()))
                .title("MacDows tray")
                .inner_size(w, h)
                .decorations(false)
                .transparent(true)
                .resizable(false)
                .shadow(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(true)
                .visible(false)
                .build();
            if let Ok(win) = built {
                if let Ok(hwnd) = win.hwnd() {
                    windows_list::register_bar(hwnd.0 as isize);
                }
                let _ = win.set_position(PhysicalPosition::new(x as i32, y as i32));
                let _ = win.show();
                let _ = win.set_focus();
            }
        });
    });
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        bring_to_front(&w);
        return;
    }
    if let Ok(w) = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("MacDows")
        .inner_size(440.0, 760.0)
        .resizable(false)
        .center()
        .build()
    {
        bring_to_front(&w);
    }
}

/// Windows refuses to focus a window from a background process, which left Settings hidden behind
/// other windows. Flashing always-on-top reliably lifts it above them.
fn bring_to_front(w: &tauri::WebviewWindow) {
    let _ = w.unminimize();
    let _ = w.show();
    let _ = w.set_always_on_top(true);
    let _ = w.set_focus();
    let _ = w.set_always_on_top(false);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let lang = app.state::<AppSettings>().0.lock().unwrap().language.clone();
    let (settings_label, quit_label) = i18n::tray_labels(&lang);
    let settings_item = MenuItem::with_id(app, "settings", settings_label, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let restore_item = MenuItem::with_id(app, "restore_taskbar", i18n::tray_restore_label(&lang), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &restore_item, &quit_item])?;
    let mut builder = TrayIconBuilder::new().menu(&menu).tooltip("MacDows");
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => open_settings(app),
            "restore_taskbar" => disable_replace_taskbar(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn place_notch(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("notch") else { return };
    let Ok(Some(monitor)) = win.primary_monitor() else { return };
    let Ok(size) = win.outer_size() else { return };
    let mon_pos = monitor.position();
    let mon_size = monitor.size();
    let x = mon_pos.x + (mon_size.width as i32 - size.width as i32) / 2;
    let _ = win.set_position(PhysicalPosition::new(x, mon_pos.y));
}

fn place_dock(app: &tauri::AppHandle) {
    let Some(win) = app.get_webview_window("dock") else { return };
    let Ok(Some(monitor)) = win.primary_monitor() else { return };
    let Ok(size) = win.outer_size() else { return };
    let area = monitor.work_area();
    let x = area.position.x + (area.size.width as i32 - size.width as i32) / 2;
    let y = area.position.y + area.size.height as i32 - size.height as i32;
    let _ = win.set_position(PhysicalPosition::new(x, y));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    match cli::parse(&refs) {
        cli::Cli::RestoreTaskbar => {
            taskbar::restore_now();
            return;
        }
        cli::Cli::Guard(pid) => {
            taskbar::guard_main(pid);
            return;
        }
        cli::Cli::Normal => {}
    }

    tauri::Builder::default()
        // Must be the first plugin: a second launch opens Settings in the running app and exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| open_settings(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppSettings(Mutex::new(settings::load())))
        .manage(updates::UpdateState::default())
        .invoke_handler(tauri::generate_handler![get_settings, set_setting, windows_list::list_windows, windows_list::focus_window, media::get_media, media::media_toggle, media::media_next, media::media_prev, media::media_seek, volume::get_volume, volume::set_volume, updates::get_update_status, updates::check_for_update, updates::install_update, appearance::get_system_accent_color, wallpaper::get_wallpaper_backdrop, taskbar::get_system_status, taskbar::shell_action, toggle_tray_panel, get_tray_items, activate_tray_item, close_tray_panel, launcher::launcher_apps, launcher::launcher_state, launcher::launch_app, launcher::toggle_pin, launcher::open_location, launcher::power_action, toggle_start_menu, toggle_search_menu, report_dock_top, close_popup, search::search_all, search::open_file, search::open_setting, search::open_web_search])
        .setup(|app| {
            for label in ["notch", "dock"] {
                if let Some(hwnd) = app.get_webview_window(label).and_then(|w| w.hwnd().ok()) {
                    windows_list::register_bar(hwnd.0 as isize);
                }
            }
            build_tray(app.handle())?;
            place_notch(app.handle());
            place_dock(app.handle());
            fullscreen::start(app.handle().clone());
            windows_list::start(app.handle().clone());
            media::start(app.handle().clone());
            visibility::start(app.handle().clone());
            updates::start(app.handle().clone());
            keyhook::start(app.handle().clone());
            launcher::warm_up();
            search::warm_up();
            start_taskbar_takeover(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Normal exit (tray Quit, update, shutdown): give the taskbar back. A crash is covered by the guard.
            if let tauri::RunEvent::Exit = event {
                keyhook::set_enabled(false);
                taskbar::restore_now();
            }
        });
}
