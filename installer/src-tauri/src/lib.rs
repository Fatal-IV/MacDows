pub mod install;
pub mod logic;

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![install::install_state, install::start_install, install::launch_app])
        .run(tauri::generate_context!())
        .expect("error while running the MacDows installer");
}
