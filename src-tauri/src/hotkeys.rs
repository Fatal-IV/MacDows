//! Win+1..9 start or focus the pinned apps of the dock (Win+Shift+number starts a new window).
//!
//! Windows has already registered these combinations for its own taskbar (`RegisterHotKey` is refused with
//! "hotkey already registered", checked on this PC), so a low-level keyboard hook claims them instead, the way
//! Nectar does. The hook only looks at the Win key and the digits 1-9; everything else passes through untouched,
//! and the hook does no work itself: it posts an event and returns.
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, AtomicUsize, Ordering};
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_LWIN, VK_RWIN, VK_SHIFT};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, PostThreadMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// An unassigned virtual key tapped after a swallowed Win+number: Explorer opens Start on a lone Win press, and
/// without a key in between a swallowed combination looks exactly like that (AutoHotkey's "menu mask key").
const MASK_VK: u16 = 0xE8;
/// Tags the mask key tap (`dwExtraInfo`) so the hook can tell its own tap from keys other programs send.
const OWN_TAG: usize = 0x4D41_434F;

/// Top-row digit keys `1`..`9` to a dock slot (0..8); the flag is "new window" (Shift held).
pub fn shortcut_for(vk: u32, shift: bool) -> Option<(usize, bool)> {
    (0x31..=0x39).contains(&vk).then(|| ((vk - 0x31) as usize, shift))
}

/// A slot with no pinned app behind it is left to Windows.
pub fn should_swallow(index: usize, pinned_count: usize) -> bool {
    index < pinned_count
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Decision {
    /// Keep the key from reaching Windows and other programs.
    pub swallow: bool,
    /// Slot and "new window" flag to hand to the dock.
    pub fire: Option<(usize, bool)>,
    /// Tap the mask key so releasing Win does not open Start.
    pub mask: bool,
}

/// What to do with one key event. `held` remembers the digit that is down (auto-repeat sends key-down again;
/// only the first counts, and the matching key-up is swallowed too).
pub fn decide(win_down: bool, vk: u32, is_down: bool, shift: bool, held: &mut u8, pinned_count: usize) -> Decision {
    let pass = Decision::default();
    if !win_down {
        return pass;
    }
    let Some((index, new_window)) = shortcut_for(vk, shift) else { return pass };
    if is_down {
        if !should_swallow(index, pinned_count) {
            return pass;
        }
        if *held == vk as u8 {
            return Decision { swallow: true, ..pass }; // auto-repeat
        }
        *held = vk as u8;
        return Decision { swallow: true, fire: Some((index, new_window)), mask: true };
    }
    if *held == vk as u8 {
        *held = 0;
        return Decision { swallow: true, ..pass };
    }
    pass
}

// ---------------------------------------------------------------------------------------------
// The hook
// ---------------------------------------------------------------------------------------------

static APP: OnceLock<AppHandle> = OnceLock::new();
static WIN_DOWN: AtomicBool = AtomicBool::new(false);
static HELD: AtomicU8 = AtomicU8::new(0);
/// How many pinned apps the dock shows; 0 while the dock is not in charge (Windows taskbar visible).
static PINNED_COUNT: AtomicUsize = AtomicUsize::new(0);
/// Thread id of the hook's message loop; 0 while the hook is off.
static HOOK_THREAD: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Serialize)]
struct Shortcut {
    index: usize,
    new_instance: bool,
}

fn shift_down() -> bool {
    unsafe { (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0 }
}

fn tap_mask_key() {
    let key = |flags| INPUT { r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VIRTUAL_KEY(MASK_VK), wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: OWN_TAG } } };
    let inputs = [key(Default::default()), key(KEYEVENTF_KEYUP)];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = unsafe { *(lparam.0 as *const KBDLLHOOKSTRUCT) };
        // The mask key tap below is injected by this hook and must not come back in; keys other programs send count.
        let own = kb.flags.0 & LLKHF_INJECTED.0 != 0 && kb.dwExtraInfo == OWN_TAG;
        if !own {
            let is_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
            let is_up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;
            let vk = kb.vkCode;
            if vk == VK_LWIN.0 as u32 || vk == VK_RWIN.0 as u32 {
                // A new Win press starts a fresh combination, even if a digit's key-up got lost.
                WIN_DOWN.store(is_down, Ordering::Relaxed);
                HELD.store(0, Ordering::Relaxed);
            } else if is_down || is_up {
                let mut held = HELD.load(Ordering::Relaxed);
                let d = decide(WIN_DOWN.load(Ordering::Relaxed), vk, is_down, shift_down(), &mut held, PINNED_COUNT.load(Ordering::Relaxed));
                HELD.store(held, Ordering::Relaxed);
                if let Some((index, new_instance)) = d.fire {
                    if d.mask {
                        tap_mask_key();
                    }
                    if let Some(app) = APP.get().cloned() {
                        // Never wait for the webview inside the hook.
                        tauri::async_runtime::spawn(async move {
                            let _ = app.emit_to("dock", "dock-shortcut", Shortcut { index, new_instance });
                        });
                    }
                }
                if d.swallow {
                    return LRESULT(1);
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn run_hook() {
    unsafe {
        HOOK_THREAD.store(GetCurrentThreadId(), Ordering::SeqCst);
        let hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("[hotkeys] the keyboard hook could not be installed: {e}");
                HOOK_THREAD.store(0, Ordering::SeqCst);
                return;
            }
        };
        eprintln!("[hotkeys] Win+1..9 claimed through a keyboard hook (RegisterHotKey is refused by Windows)");
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = UnhookWindowsHookEx(hook);
        HOOK_THREAD.store(0, Ordering::SeqCst);
        WIN_DOWN.store(false, Ordering::Relaxed);
        HELD.store(0, Ordering::Relaxed);
    }
}

/// Turns the shortcuts on or off. Off means no hook at all.
pub fn apply(app: &AppHandle, enabled: bool) {
    let _ = APP.set(app.clone());
    let running = HOOK_THREAD.load(Ordering::SeqCst);
    if enabled && running == 0 {
        std::thread::spawn(run_hook);
    } else if !enabled && running != 0 {
        unsafe {
            let _ = PostThreadMessageW(running, WM_QUIT, WPARAM(0), LPARAM(0));
        }
    }
}

/// The dock reports how many pinned apps it shows (0 while it is not in charge of the taskbar).
#[tauri::command]
pub fn set_dock_pinned_count(count: usize) {
    PINNED_COUNT.store(count, Ordering::Relaxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIN_ONE: u32 = 0x31;

    #[test]
    fn the_digits_one_to_nine_are_slots_and_shift_means_a_new_window() {
        assert_eq!(shortcut_for(0x31, false), Some((0, false)));
        assert_eq!(shortcut_for(0x39, true), Some((8, true)));
        assert_eq!(shortcut_for(0x30, false), None);
        assert_eq!(shortcut_for(0x41, false), None);
    }

    #[test]
    fn an_empty_slot_is_left_to_windows() {
        assert!(should_swallow(0, 1));
        assert!(!should_swallow(1, 1));
        assert!(!should_swallow(0, 0));
    }

    #[test]
    fn win_plus_one_fires_once_and_masks_start() {
        let mut held = 0;
        let d = decide(true, WIN_ONE, true, false, &mut held, 3);
        assert_eq!(d, Decision { swallow: true, fire: Some((0, false)), mask: true });
        assert_eq!(held, WIN_ONE as u8);
    }

    #[test]
    fn auto_repeat_is_swallowed_without_firing_again() {
        let mut held = 0;
        decide(true, WIN_ONE, true, false, &mut held, 3);
        let repeat = decide(true, WIN_ONE, true, false, &mut held, 3);
        assert_eq!(repeat, Decision { swallow: true, fire: None, mask: false });
    }

    #[test]
    fn the_key_up_of_a_swallowed_digit_is_swallowed_and_resets() {
        let mut held = 0;
        decide(true, WIN_ONE, true, false, &mut held, 3);
        let up = decide(true, WIN_ONE, false, false, &mut held, 3);
        assert_eq!(up, Decision { swallow: true, fire: None, mask: false });
        assert_eq!(held, 0);
        // Pressing it again fires again.
        assert!(decide(true, WIN_ONE, true, false, &mut held, 3).fire.is_some());
    }

    #[test]
    fn shift_asks_for_a_new_window() {
        let mut held = 0;
        assert_eq!(decide(true, 0x32, true, true, &mut held, 3).fire, Some((1, true)));
    }

    #[test]
    fn without_win_or_for_other_keys_or_empty_slots_everything_passes() {
        let mut held = 0;
        assert_eq!(decide(false, WIN_ONE, true, false, &mut held, 3), Decision::default());
        assert_eq!(decide(true, 0x41, true, false, &mut held, 3), Decision::default());
        assert_eq!(decide(true, 0x35, true, false, &mut held, 3), Decision::default(), "Win+5 with 3 pins");
        assert_eq!(decide(true, WIN_ONE, true, false, &mut held, 0), Decision::default(), "dock not in charge");
        assert_eq!(held, 0);
    }

    #[test]
    fn a_digit_up_that_was_never_swallowed_passes() {
        let mut held = 0;
        assert_eq!(decide(true, WIN_ONE, false, false, &mut held, 3), Decision::default());
    }

    /// Run by hand: `cargo test win_number_registration -- --ignored --nocapture`. Windows refuses to let a program
    /// register Win+1..9 ("hotkey already registered"), which is why the keyboard hook is used.
    #[test]
    #[ignore]
    fn win_number_registration() {
        use windows::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey, MOD_NOREPEAT, MOD_WIN};
        for n in 1..=9u32 {
            let vk = 0x30 + n;
            let result = unsafe { RegisterHotKey(None, 100 + n as i32, MOD_WIN | MOD_NOREPEAT, vk) };
            println!("Win+{n}: {}", match &result { Ok(()) => "registered".to_string(), Err(e) => format!("refused ({e})") });
            if result.is_ok() {
                unsafe {
                    let _ = UnregisterHotKey(None, 100 + n as i32);
                }
            }
        }
    }
}
