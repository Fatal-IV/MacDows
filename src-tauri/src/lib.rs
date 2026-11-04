mod i18n;
mod cli;
mod taskbar;
mod taskbar_plan;
mod system_theme;
mod tray_panel;
mod updates;
mod visualizer;
mod volume;
mod quick_settings;
mod notifications;
mod secure_store;
mod shelf;
mod bluetooth;
mod wifi;
mod visibility;
mod wallpaper;
mod media;
mod windows_list;
mod pinned_apps;
mod dock_menu;
mod custom_icons;
mod drives;
mod hotkeys;
mod preview;
mod app_catalog;
mod appearance;
mod fullscreen;
mod settings;
mod island_settings;
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
    let mut updated: settings::Settings = serde_json::from_value(json).map_err(|e| e.to_string())?;
    updated.island.normalize();
    settings::save(&updated).map_err(|e| e.to_string())?;
    *guard = updated.clone();
    drop(guard);
    let _ = app.emit("settings-changed", updated.clone());
    match key.as_str() {
        "replace_taskbar" => apply_replace_taskbar(&app, updated.replace_taskbar),
        "start_with_windows" => taskbar::set_autostart(updated.start_with_windows),
        "dock" => hotkeys::apply(&app, updated.dock.win_number),
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
                // Windows's Start and Search stay, in the colors of the MacDows theme while it runs.
                system_theme::set_enabled(true);
            }
        } else {
            // Puts the user's own personalization colors back (and then the taskbar).
            system_theme::set_enabled(false);
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
        .inner_size(860.0, 620.0)
        .min_inner_size(740.0, 500.0)
        .decorations(false)
        .shadow(true)
        .center()
        .build()
    {
        bring_to_front(&w);
    }
}

/// The notch's quick shortcut for MacDows's own settings. The window is built on a worker thread, not the main one.
#[tauri::command]
fn open_macdows_settings(app: AppHandle) {
    std::thread::spawn(move || open_settings(&app));
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
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .on_menu_event(|app, event| dock_menu::on_menu_event(app, event.id().as_ref()))
        .manage(AppSettings(Mutex::new(settings::load())))
        .manage(updates::UpdateState::default())
        .invoke_handler(tauri::generate_handler![get_settings, set_setting, windows_list::list_windows, windows_list::focus_window, pinned_apps::list_pinned, pinned_apps::launch_pinned, pinned_apps::set_pinned_order, dock_menu::dock_menu, custom_icons::get_custom_icons, hotkeys::set_dock_pinned_count, app_catalog::get_catalog, app_catalog::pin_catalog_app, app_catalog::close_add_app_panel, preview::show_preview, preview::preview_hover, preview::preview_items, preview::close_preview, preview::preview_activate, preview::preview_close_window, pinned_apps::launch_new_app, drives::list_drives, drives::open_drive, media::get_media, media::media_toggle, media::media_next, media::media_prev, media::media_seek, media::get_media_sources, media::select_media_source, volume::get_volume, volume::set_volume, quick_settings::get_quick_settings, quick_settings::set_quick_setting, quick_settings::get_brightness, quick_settings::quick_launch, open_macdows_settings, notifications::get_notifications, notifications::remove_notification, notifications::clear_notifications, notifications::get_notification_history, notifications::clear_notification_history, bluetooth::get_bluetooth_devices, bluetooth::set_bluetooth_connection, wifi::wifi_state, wifi::wifi_scan, shelf::get_shelf, shelf::add_to_shelf, shelf::remove_from_shelf, shelf::clear_shelf, shelf::open_shelf_item, shelf::reveal_shelf_item, quick_settings::set_brightness, updates::get_update_status, updates::check_for_update, updates::install_update, appearance::get_system_accent_color, wallpaper::get_wallpaper_backdrop, taskbar::get_system_status, taskbar::shell_action, toggle_tray_panel, get_tray_items, activate_tray_item, close_tray_panel, system_theme::sync_system_theme])
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
            pinned_apps::start(app.handle().clone());
            drives::start(app.handle().clone());
            {
                let on = app.state::<AppSettings>().0.lock().unwrap().dock.win_number;
                hotkeys::apply(app.handle(), on);
            }
            media::start(app.handle().clone());
            notifications::start(app.handle().clone());
            visualizer::start(app.handle().clone());
            visibility::start(app.handle().clone());
            updates::start(app.handle().clone());
            start_taskbar_takeover(app.handle());
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            // Normal exit (tray Quit, update, shutdown): give the taskbar back. A crash is covered by the guard.
            if let tauri::RunEvent::Exit = event {
                taskbar::restore_now();
            }
        });
}
