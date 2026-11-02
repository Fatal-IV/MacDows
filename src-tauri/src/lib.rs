mod i18n;
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
    json.as_object_mut().ok_or("settings not an object")?.insert(key, value);
    let updated: settings::Settings = serde_json::from_value(json).map_err(|e| e.to_string())?;
    settings::save(&updated).map_err(|e| e.to_string())?;
    *guard = updated.clone();
    let _ = app.emit("settings-changed", updated.clone());
    Ok(updated)
}

fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("MacDows")
        .inner_size(440.0, 400.0)
        .resizable(false)
        .center()
        .build();
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let lang = app.state::<AppSettings>().0.lock().unwrap().language.clone();
    let (settings_label, quit_label) = i18n::tray_labels(&lang);
    let settings_item = MenuItem::with_id(app, "settings", settings_label, true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;
    let mut builder = TrayIconBuilder::new().menu(&menu).tooltip("MacDows");
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder
        .on_menu_event(|app, event| match event.id().as_ref() {
            "settings" => open_settings(app),
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
    tauri::Builder::default()
        // Must be the first plugin: a second launch opens Settings in the running app and exits.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| open_settings(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppSettings(Mutex::new(settings::load())))
        .manage(updates::UpdateState::default())
        .invoke_handler(tauri::generate_handler![get_settings, set_setting, windows_list::list_windows, windows_list::focus_window, media::get_media, media::media_toggle, media::media_next, media::media_prev, updates::get_update_status, updates::check_for_update, updates::install_update])
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
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
