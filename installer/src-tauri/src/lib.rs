pub mod logic;

#[tauri::command]
fn install_state() -> Result<String, String> {
    Err("not implemented".into())
}

#[tauri::command]
fn start_install() -> Result<(), String> {
    Err("not implemented".into())
}

#[tauri::command]
fn launch_app() -> Result<(), String> {
    Err("not implemented".into())
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![install_state, start_install, launch_app])
        .run(tauri::generate_context!())
        .expect("error while running the MacDows installer");
}
