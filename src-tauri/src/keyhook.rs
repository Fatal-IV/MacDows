//! Makes the Windows key open MacDows's Start menu and Win+S its search, instead of Windows's.
//!
//! A low-level keyboard hook sees every key. The rules are deliberately narrow so nothing else breaks:
//! * a Windows key that is pressed and released with no other key in between opens Start (Windows's own Start is
//!   cancelled by sending a harmless key before the Windows key is released);
//! * Win+S with no Ctrl, Shift or Alt held opens search, and the S never reaches Windows;
//! * every other combination (Win+E, Win+Shift+S, Win+Tab, ...) passes through untouched.
//! The hook only acts while MacDows has taken over the taskbar.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
};

/// Marks the keys MacDows sends itself, so the hook does not react to them.
const OWN_EVENT: usize = 0x4D44_5753;
const VK_CANCEL_START: u16 = 0xE8;
const VK_S: u32 = 0x53;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Open {
    Start,
    Search,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Do not pass this key on to Windows.
    pub swallow: bool,
    pub open: Option<Open>,
    /// Send a harmless key first, so Windows does not open its own Start when the Windows key comes up.
    pub cancel_start: bool,
}

fn is_win(vk: u32) -> bool {
    vk == 0x5B || vk == 0x5C
}

fn is_modifier(vk: u32) -> bool {
    matches!(vk, 0x10 | 0x11 | 0x12 | 0xA0..=0xA5)
}

/// The key rules, free of any Windows calls so they can be tested.
#[derive(Default)]
pub struct Machine {
    win_down: bool,
    /// Another key was used while the Windows key was held: it is a shortcut, not a Start tap.
    combo: bool,
    /// We handled a key of the combination ourselves, so Windows still has to be kept from opening Start.
    handled: bool,
    modifiers_down: u32,
    swallowed_up: Vec<u32>,
}

impl Machine {
    pub fn event(&mut self, vk: u32, down: bool) -> Outcome {
        let mut out = Outcome::default();
        if is_win(vk) {
            if down {
                if !self.win_down {
                    self.win_down = true;
                    self.combo = false;
                    self.handled = false;
                }
            } else if self.win_down {
                self.win_down = false;
                if !self.combo {
                    out.open = Some(Open::Start);
                    out.cancel_start = true;
                } else if self.handled {
                    out.cancel_start = true;
                }
            }
            return out;
        }
        if is_modifier(vk) {
            if down {
                // Auto-repeat sends many downs; count each physical key once through a floor at zero on release.
                self.modifiers_down = self.modifiers_down.saturating_add(1).min(8);
            } else {
                self.modifiers_down = self.modifiers_down.saturating_sub(1);
            }
            if down && self.win_down {
                self.combo = true;
            }
            return out;
        }
        if down {
            if !self.win_down {
                // A fresh press of a key whose release we were still waiting to swallow: that release was lost.
                self.swallowed_up.retain(|k| *k != vk);
            }
            if self.win_down {
                self.combo = true;
                if vk == VK_S && self.modifiers_down == 0 {
                    self.handled = true;
                    self.swallowed_up.push(vk);
                    out.swallow = true;
                    out.open = Some(Open::Search);
                }
            }
        } else if let Some(pos) = self.swallowed_up.iter().position(|k| *k == vk) {
            self.swallowed_up.remove(pos);
            out.swallow = true;
        }
        out
    }
}

static MACHINE: Mutex<Option<Machine>> = Mutex::new(None);
static ENABLED: AtomicBool = AtomicBool::new(false);
static APP: OnceLock<AppHandle> = OnceLock::new();
/// The dock's top edge in CSS pixels of its window, reported by the dock (the popups sit above it).
static DOCK_TOP_BITS: AtomicU64 = AtomicU64::new(0);

pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::SeqCst);
    if !on {
        *MACHINE.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

pub fn set_dock_top(top: f64) {
    DOCK_TOP_BITS.store(top.to_bits(), Ordering::SeqCst);
}

fn dock_top() -> f64 {
    let top = f64::from_bits(DOCK_TOP_BITS.load(Ordering::SeqCst));
    if top.is_finite() && top > 0.0 { top } else { 63.0 }
}

fn key(vk: u16, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: OWN_EVENT,
            },
        },
    }
}

fn cancel_windows_start() {
    let inputs = [key(VK_CANCEL_START, false), key(VK_CANCEL_START, true)];
    unsafe {
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

fn open(kind: Open) {
    let Some(app) = APP.get().cloned() else { return };
    // Off the hook: it has to return within a fraction of a second or Windows removes it.
    std::thread::spawn(move || match kind {
        Open::Start => crate::open_start_from_key(&app, dock_top()),
        Open::Search => crate::open_search_from_key(&app, dock_top()),
    });
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && ENABLED.load(Ordering::SeqCst) {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        if info.dwExtraInfo != OWN_EVENT {
            let down = matches!(wparam.0 as u32, 0x100 | 0x104);
            let outcome = {
                let mut guard = MACHINE.lock().unwrap_or_else(|e| e.into_inner());
                guard.get_or_insert_with(Machine::default).event(info.vkCode, down)
            };
            if outcome.cancel_start {
                cancel_windows_start();
            }
            if let Some(kind) = outcome.open {
                open(kind);
            }
            if outcome.swallow {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(Some(HHOOK::default()), code, wparam, lparam) }
}

/// Installs the hook on its own thread (a hook needs a message loop). It does nothing until `set_enabled(true)`.
pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    std::thread::spawn(|| unsafe {
        let Ok(_hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0) else { return };
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

    const WIN: u32 = 0x5B;
    const E: u32 = 0x45;
    const SHIFT: u32 = 0xA0;
    const CTRL: u32 = 0xA2;

    fn run(m: &mut Machine, steps: &[(u32, bool)]) -> Vec<Outcome> {
        steps.iter().map(|(vk, down)| m.event(*vk, *down)).collect()
    }

    #[test]
    fn a_lone_windows_key_tap_opens_start_and_cancels_the_windows_one() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (WIN, false)]);
        assert_eq!(out[0], Outcome::default());
        assert_eq!(out[1], Outcome { swallow: false, open: Some(Open::Start), cancel_start: true });
    }

    #[test]
    fn holding_the_windows_key_with_auto_repeat_still_opens_start_once() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (WIN, true), (WIN, true), (WIN, false)]);
        assert_eq!(out.iter().filter(|o| o.open == Some(Open::Start)).count(), 1);
    }

    #[test]
    fn other_win_shortcuts_pass_through_untouched() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (E, true), (E, false), (WIN, false)]);
        assert!(out.iter().all(|o| *o == Outcome::default()), "{out:?}");
    }

    #[test]
    fn win_s_opens_search_swallows_the_s_and_cancels_start() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (VK_S, true), (VK_S, false), (WIN, false)]);
        assert_eq!(out[1], Outcome { swallow: true, open: Some(Open::Search), cancel_start: false });
        assert_eq!(out[2], Outcome { swallow: true, open: None, cancel_start: false });
        assert_eq!(out[3], Outcome { swallow: false, open: None, cancel_start: true });
    }

    #[test]
    fn win_shift_s_and_ctrl_win_s_are_left_alone() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (SHIFT, true), (VK_S, true), (VK_S, false), (SHIFT, false), (WIN, false)]);
        assert!(out.iter().all(|o| *o == Outcome::default()), "{out:?}");
        let mut m = Machine::default();
        let out = run(&mut m, &[(CTRL, true), (WIN, true), (VK_S, true), (VK_S, false), (WIN, false), (CTRL, false)]);
        assert!(out.iter().all(|o| *o == Outcome::default()), "{out:?}");
    }

    #[test]
    fn a_modifier_pressed_with_the_windows_key_means_it_is_a_shortcut_not_a_start_tap() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (CTRL, true), (CTRL, false), (WIN, false)]);
        assert!(out.iter().all(|o| o.open.is_none()), "{out:?}");
    }

    #[test]
    fn s_without_the_windows_key_is_never_touched() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(VK_S, true), (VK_S, false)]);
        assert!(out.iter().all(|o| *o == Outcome::default()));
    }

    #[test]
    fn a_lost_key_release_never_leaves_a_later_s_stuck_down() {
        let mut m = Machine::default();
        // Win+S, and the release of S never arrives (for example the hook was off for a moment).
        run(&mut m, &[(WIN, true), (VK_S, true), (WIN, false)]);
        // Typing S again later: both its press and its release must reach Windows.
        let out = run(&mut m, &[(VK_S, true), (VK_S, false)]);
        assert_eq!(out[0], Outcome::default());
        assert_eq!(out[1], Outcome::default());
    }

    #[test]
    fn the_release_of_a_swallowed_s_is_swallowed_even_after_the_windows_key_came_up() {
        let mut m = Machine::default();
        let out = run(&mut m, &[(WIN, true), (VK_S, true), (WIN, false), (VK_S, false)]);
        assert!(out[3].swallow, "{out:?}");
    }
}
