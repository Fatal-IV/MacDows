use tauri::{Manager, PhysicalPosition};

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
        .setup(|app| {
            place_notch(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
