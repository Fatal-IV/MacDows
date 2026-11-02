mod settings;
use std::sync::Mutex;
use tauri::{Manager, PhysicalPosition, State};

struct AppSettings(Mutex<settings::Settings>);

#[tauri::command]
fn get_settings(state: State<AppSettings>) -> settings::Settings {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
fn set_setting(state: State<AppSettings>, key: String, value: serde_json::Value) -> Result<settings::Settings, String> {
    let mut guard = state.0.lock().unwrap();
    let mut json = serde_json::to_value(&*guard).map_err(|e| e.to_string())?;
    json.as_object_mut().ok_or("settings not an object")?.insert(key, value);
    let updated: settings::Settings = serde_json::from_value(json).map_err(|e| e.to_string())?;
    settings::save(&updated).map_err(|e| e.to_string())?;
    *guard = updated.clone();
    Ok(updated)
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppSettings(Mutex::new(settings::load())))
        .invoke_handler(tauri::generate_handler![get_settings, set_setting])
        .setup(|app| {
            place_notch(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
