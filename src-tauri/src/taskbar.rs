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
            set_appbar_state(t, if on { current | ABS_AUTOHIDE } else { current & !ABS_AUTOHIDE });
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

/// Puts the taskbar back to the state recorded before MacDows touched it (used by the exit hook, the CLI flag and the guard).
pub fn restore_now() {
    crate::taskbar_plan::restore(&WinBackend, &FileStore::default());
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

fn run_steps(steps: &[KeyStep]) {
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
    pub hidden_icons_available: bool,
    pub taskbar_hidden: bool,
}

fn current_status() -> SystemStatus {
    let mut power = SYSTEM_POWER_STATUS::default();
    let battery_percent = unsafe { GetSystemPowerStatus(&mut power) }.ok().and_then(|_| battery_percent_from(power.BatteryLifePercent));
    let taskbar_hidden = WinBackend.window_hidden();
    let available = FileStore::default().load().map_or(false, |s| hidden_icons_available(&s, taskbar_hidden));
    SystemStatus { battery_percent, hidden_icons_available: available, taskbar_hidden }
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
        "hidden_icons" => {
            if !current_status().hidden_icons_available {
                return Err("hidden_icons_unavailable".into());
            }
            ShellAction::HiddenIcons
        }
        other => return Err(format!("unknown action {other}")),
    };
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
        TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: pid }
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
