// No console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebView2 must exist before the window is built, so this runs first and needs no window itself.
    if !macdows_setup_lib::launcher::prepare(&macdows_setup_lib::install::system_locale()) {
        return;
    }
    macdows_setup_lib::run()
}
