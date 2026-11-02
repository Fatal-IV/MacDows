mod i18n;
mod cli;
mod taskbar;
mod taskbar_plan;
mod updates;
mod visibility;
mod media;
mod windows_list;
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
            }
        } else {
            taskbar::restore_now();
        }
        // The work area changes a moment after the taskbar does.
        std::thread::sleep(std::time::Duration::from_millis(1500));
        place_dock(&app);
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
        }
    });
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("MacDows")
        .inner_size(440.0, 660.0)
        .resizable(false)
        .center()
        .build();
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
        .invoke_handler(tauri::generate_handler![get_settings, set_setting, windows_list::list_windows, windows_list::focus_window, media::get_media, media::media_toggle, media::media_next, media::media_prev, updates::get_update_status, updates::check_for_update, updates::install_update, taskbar::get_system_status, taskbar::shell_action])
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
