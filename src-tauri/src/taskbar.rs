//! The real Windows side of the taskbar takeover: appbar state, window visibility, the state file, the
//! shell shortcuts, the Explorer-restart watcher. Decisions live in `taskbar_plan.rs`.
use crate::taskbar_plan::{hidden_icons_available, key_steps, state_from_json, state_to_json, Backend, KeyStep, ShellAction, Store, TaskbarState};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;
use std::time::Duration;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, TreeScope_Descendants};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY,
};
use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_GETSTATE, ABM_SETSTATE, APPBARDATA};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetMessageW, IsWindowVisible, RegisterClassW,
    RegisterWindowMessageW, ShowWindow, TranslateMessage, MSG, SW_HIDE, SW_SHOW, WINDOW_EX_STYLE, WNDCLASSW, WS_POPUP,
};

const ABS_AUTOHIDE: usize = 1;

/// Names of the "show hidden icons" tray item. Other Windows languages are not known, so the hidden-icons
/// button is simply not offered there.
pub fn is_chevron_name(name: &str) -> bool {
    let n = name.trim().to_lowercase();
    n == "show hidden icons" || n == "gizli simgeleri göster"
}

/// `SYSTEM_POWER_STATUS.BatteryLifePercent`: 0..=100, 255 means unknown.
pub fn battery_percent_from(raw: u8) -> Option<u8> {
    (raw <= 100).then_some(raw)
}

// ---------------------------------------------------------------------------------------------
// State file
// ---------------------------------------------------------------------------------------------
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn at(path: PathBuf) -> Self {
        FileStore { path }
    }
}

impl Default for FileStore {
    fn default() -> Self {
        let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
        FileStore::at(base.join("MacDows").join("taskbar-state.json"))
    }
}

impl Store for FileStore {
    fn load(&self) -> Option<TaskbarState> {
        state_from_json(&std::fs::read_to_string(&self.path).ok()?)
    }

    /// Written through a temp file and renamed, so a crash never leaves half a file.
    fn save(&self, state: &TaskbarState) -> std::io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, state_to_json(state))?;
        std::fs::rename(&tmp, &self.path)
    }

    fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

// ---------------------------------------------------------------------------------------------
// Windows backend
// ---------------------------------------------------------------------------------------------
pub struct WinBackend;

fn tray_window() -> Option<HWND> {
    unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()).ok() }
}

fn appbar_state(tray: HWND) -> usize {
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: tray, ..Default::default() };
    unsafe { SHAppBarMessage(ABM_GETSTATE, &mut data) }
}

fn set_appbar_state(tray: HWND, state: usize) {
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: tray, lParam: LPARAM(state as isize), ..Default::default() };
    unsafe {
        SHAppBarMessage(ABM_SETSTATE, &mut data);
    }
}

/// UI Automation needs its own COM apartment, so it runs on a short-lived thread.
fn chevron_present_uia(tray: HWND) -> bool {
    let raw = tray.0 as isize;
    std::thread::spawn(move || unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let tray = HWND(raw as *mut _);
        let Ok(automation) = CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER) else { return false };
        let Ok(root) = automation.ElementFromHandle(tray) else { return false };
        let Ok(condition) = automation.CreateTrueCondition() else { return false };
        let Ok(all) = root.FindAll(TreeScope_Descendants, &condition) else { return false };
        let count = all.Length().unwrap_or(0);
        for i in 0..count {
            if let Ok(element) = all.GetElement(i) {
                if let Ok(name) = element.CurrentName() {
                    if is_chevron_name(&name.to_string()) {
                        return true;
                    }
                }
            }
        }
        false
    })
    .join()
    .unwrap_or(false)
}

impl Backend for WinBackend {
    fn autohide(&self) -> bool {
        tray_window().map_or(false, |t| appbar_state(t) & ABS_AUTOHIDE != 0)
    }

    fn set_autohide(&self, on: bool) {
        if let Some(t) = tray_window() {
            let current = appbar_state(t);
            let wanted = if on { current | ABS_AUTOHIDE } else { current & !ABS_AUTOHIDE };
            if wanted != current {
                set_appbar_state(t, wanted);
                // Explorer re-lays the taskbar out after a state change; hiding the window too early can be undone.
                std::thread::sleep(Duration::from_millis(800));
            }
        }
    }

    fn hide_window(&self) {
        if let Some(t) = tray_window() {
            unsafe {
                let _ = ShowWindow(t, SW_HIDE);
            }
        }
    }

    fn show_window(&self) {
        if let Some(t) = tray_window() {
            unsafe {
                let _ = ShowWindow(t, SW_SHOW);
            }
        }
    }

    fn window_hidden(&self) -> bool {
        tray_window().map_or(false, |t| !unsafe { IsWindowVisible(t) }.as_bool())
    }

    fn chevron_present(&self) -> bool {
        tray_window().map_or(false, chevron_present_uia)
    }
}

/// While the setting is on, hides the taskbar again whenever Explorer shows it. Checks every 2 seconds.
pub fn start_watchdog(setting_on: impl Fn() -> bool + Send + 'static) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(2));
        let visible = !WinBackend.window_hidden();
        if visible && !crate::tray_panel::is_busy() && crate::taskbar_plan::watchdog_should_rehide(setting_on(), FileStore::default().load().as_ref(), visible) {
            WinBackend.hide_window();
        }
    });
}

/// Puts the taskbar back to the state recorded before MacDows touched it (used by the exit hook, the CLI flag and the guard).
pub fn restore_now() {
    // The personalization colors MacDows set for the Windows menus go back first, on every path that ends the takeover.
    crate::system_theme::restore();
    crate::taskbar_plan::restore(&WinBackend, &FileStore::default());
    settle_visible();
}

/// A second look a moment later: the installer, the exit hook and the guard can restore at the same time and
/// Explorer may undo one of them. With no owner left, a hidden taskbar is shown again.
fn settle_visible() {
    std::thread::sleep(Duration::from_millis(1500));
    if crate::taskbar_plan::needs_final_show(FileStore::default().load().as_ref(), WinBackend.window_hidden()) {
        WinBackend.show_window();
    }
}

/// Hides the taskbar for this process. Returns false when the state could not be saved (nothing was hidden).
pub fn hide_now() -> bool {
    crate::taskbar_plan::hide(&WinBackend, &FileStore::default(), std::process::id()).is_ok()
}

// ---------------------------------------------------------------------------------------------
// Shell shortcuts and status
// ---------------------------------------------------------------------------------------------
fn key_input(vk: u8, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk as u16),
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

pub(crate) fn run_steps(steps: &[KeyStep]) {
    const VK_LWIN: u8 = 0x5B;
    for step in steps {
        match *step {
            KeyStep::Wait { ms } => std::thread::sleep(Duration::from_millis(ms as u64)),
            KeyStep::Chord { win, key } => {
                let mut inputs = Vec::new();
                if win {
                    inputs.push(key_input(VK_LWIN, false));
                }
                inputs.push(key_input(key, false));
                inputs.push(key_input(key, true));
                if win {
                    inputs.push(key_input(VK_LWIN, true));
                }
                unsafe {
                    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
                }
            }
        }
    }
}

#[derive(Serialize)]
pub struct SystemStatus {
    pub battery_percent: Option<u8>,
    /// Plugged in (AC power).
    pub battery_charging: bool,
    pub hidden_icons_available: bool,
    pub taskbar_hidden: bool,
}

fn current_status() -> SystemStatus {
    let mut power = SYSTEM_POWER_STATUS::default();
    let battery_percent = unsafe { GetSystemPowerStatus(&mut power) }.ok().and_then(|_| battery_percent_from(power.BatteryLifePercent));
    let taskbar_hidden = WinBackend.window_hidden();
    let available = FileStore::default().load().map_or(false, |s| hidden_icons_available(&s, taskbar_hidden));
    SystemStatus { battery_percent, battery_charging: power.ACLineStatus == 1, hidden_icons_available: available, taskbar_hidden }
}

#[tauri::command]
pub fn get_system_status() -> SystemStatus {
    current_status()
}

#[tauri::command]
pub fn shell_action(action: String) -> Result<(), String> {
    let action = match action.as_str() {
        "notifications" => ShellAction::Notifications,
        "quick_settings" => ShellAction::QuickSettings,
        "start" => ShellAction::Start,
        "search" => ShellAction::Search,
        "show_desktop" => ShellAction::ShowDesktop,
        "hidden_icons" => {
            if !current_status().hidden_icons_available {
                return Err("hidden_icons_unavailable".into());
            }
            ShellAction::HiddenIcons
        }
        other => return Err(format!("unknown action {other}")),
    };
    // The click on the dock button already closed an open Start or Search; pressing the key again would reopen it.
    if matches!(action, ShellAction::Start | ShellAction::Search) && crate::visibility::shell_menu_just_closed() {
        return Ok(());
    }
    // Off the UI thread: the hidden-icons shortcut waits between its keys.
    std::thread::spawn(move || run_steps(&key_steps(action)));
    Ok(())
}

// ---------------------------------------------------------------------------------------------
// Explorer restart
// ---------------------------------------------------------------------------------------------
static ON_TASKBAR_CREATED: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
static TASKBAR_CREATED_MSG: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn watcher_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let created = TASKBAR_CREATED_MSG.load(Ordering::Relaxed);
    if created != 0 && msg == created {
        if let Some(callback) = ON_TASKBAR_CREATED.get() {
            callback();
        }
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

/// Calls `callback` every time Explorer creates a new taskbar (after it restarted). Broadcast messages only
/// reach top-level windows, so this is a hidden popup window with its own message loop.
pub fn watch_taskbar_created(callback: impl Fn() + Send + Sync + 'static) {
    let _ = ON_TASKBAR_CREATED.set(Box::new(callback));
    std::thread::spawn(|| unsafe {
        TASKBAR_CREATED_MSG.store(RegisterWindowMessageW(w!("TaskbarCreated")), Ordering::Relaxed);
        let class = w!("MacDowsTaskbarWatcher");
        let wc = WNDCLASSW { lpfnWndProc: Some(watcher_proc), lpszClassName: class, ..Default::default() };
        RegisterClassW(&wc);
        let _ = CreateWindowExW(WINDOW_EX_STYLE(0), class, w!(""), WS_POPUP, 0, 0, 0, 0, None, None, None, None);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

// ---------------------------------------------------------------------------------------------
// Guard
// ---------------------------------------------------------------------------------------------
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const DETACHED_PROCESS: u32 = 0x0000_0008;
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

fn guard_exe_path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("MacDows").join("guard").join("macdows-guard.exe")
}

/// Runs inside the guard process (`macdows-guard.exe --guard <pid>`): waits for the app to end, then restores.
pub fn guard_main(parent_pid: u32) {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{OpenProcess, WaitForSingleObject, INFINITE, PROCESS_SYNCHRONIZE};
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_SYNCHRONIZE, false, parent_pid) {
            WaitForSingleObject(handle, INFINITE);
            let _ = CloseHandle(handle);
        }
    }
    let state = FileStore::default().load();
    let ours = state.as_ref().map_or(false, |s| crate::taskbar_plan::state_belongs_to(s, parent_pid));
    if ours && crate::taskbar_plan::guard_should_restore(false, state.as_ref()) {
        restore_now();
    } else {
        // The app's own exit hook may already have cleared the state and been cut short by the installer.
        settle_visible();
    }
}

/// Starts the guard for this process. Returns false when it could not be started (the exit hook and the
/// next-start check still restore the taskbar).
pub fn start_guard() -> bool {
    use std::os::windows::process::CommandExt;
    let Ok(me) = std::env::current_exe() else { return false };
    let dest = guard_exe_path();
    let source_len = std::fs::metadata(&me).map(|m| m.len()).unwrap_or(0);
    let copy_len = std::fs::metadata(&dest).ok().map(|m| m.len());
    if crate::taskbar_plan::guard_needs_refresh(source_len, copy_len) {
        if let Some(dir) = dest.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let fresh = dest.with_extension("exe.new");
        if std::fs::copy(&me, &fresh).is_ok() {
            // Fails while an older guard still runs from the old file; the old copy then stays, which is fine.
            let _ = std::fs::rename(&fresh, &dest);
            let _ = std::fs::remove_file(&fresh);
        }
    }
    if !dest.exists() {
        return false;
    }
    std::process::Command::new(&dest)
        .args(["--guard", &std::process::id().to_string()])
        .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .is_ok()
}

// ---------------------------------------------------------------------------------------------
// Process, autostart and notice helpers
// ---------------------------------------------------------------------------------------------
/// Whether a process with this id is still running.
pub fn pid_alive(pid: u32) -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return false };
        let mut code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut code).is_ok();
        let _ = CloseHandle(handle);
        ok && code == STILL_ACTIVE
    }
}

/// Adds or removes the "start with Windows" entry. Debug builds never touch the registry.
pub fn set_autostart(on: bool) {
    if cfg!(debug_assertions) {
        return;
    }
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;
    let Ok(key) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", KEY_SET_VALUE) else {
        return;
    };
    if on {
        if let Ok(exe) = std::env::current_exe() {
            let _ = key.set_value("MacDows", &crate::taskbar_plan::run_value(&exe.to_string_lossy()));
        }
    } else {
        let _ = key.delete_value("MacDows");
    }
}

/// A plain native message box (blocks until OK). Used for the one-time notice.
pub fn show_notice(title: &str, text: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
    let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
    let (t, m) = (wide(title), wide(text));
    unsafe {
        MessageBoxW(None, PCWSTR(m.as_ptr()), PCWSTR(t.as_ptr()), MB_OK | MB_ICONINFORMATION);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::taskbar_plan::{Store, TaskbarState};
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn state(pid: u32) -> TaskbarState {
        TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: pid, hidden_secondary: false }
    }

    #[test]
    fn file_store_roundtrip_replace_clear_and_corrupt_file() {
        let dir = temp_dir("Çağrı macdows-taskbar-test");
        let store = FileStore::at(dir.join("taskbar-state.json"));
        assert_eq!(store.load(), None);
        store.save(&state(1)).unwrap();
        assert_eq!(store.load(), Some(state(1)));
        store.save(&state(2)).unwrap();
        assert_eq!(store.load(), Some(state(2)));
        let leftovers: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).collect();
        assert_eq!(leftovers.len(), 1, "no temp file may stay behind");
        std::fs::write(dir.join("taskbar-state.json"), "garbage").unwrap();
        assert_eq!(store.load(), None);
        store.clear();
        assert!(!dir.join("taskbar-state.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_hidden_icons_chevron_is_recognized_by_its_english_and_turkish_name() {
        assert!(is_chevron_name("Show Hidden Icons"));
        assert!(is_chevron_name("Gizli Simgeleri Göster"));
        assert!(is_chevron_name("  show hidden icons "));
        assert!(!is_chevron_name("Clock"));
        assert!(!is_chevron_name(""));
    }

    #[test]
    fn battery_percent_maps_unknown_to_none() {
        assert_eq!(battery_percent_from(100), Some(100));
        assert_eq!(battery_percent_from(0), Some(0));
        assert_eq!(battery_percent_from(255), None);
        assert_eq!(battery_percent_from(101), None);
    }
}
