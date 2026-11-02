//! Taskbar takeover decisions. Pure functions (unit tested); the Windows calls live in `taskbar.rs`.
use serde::{Deserialize, Serialize};

pub const STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskbarState {
    pub version: u32,
    pub hidden_by_macdows: bool,
    /// Whether the Windows taskbar had auto-hide on before MacDows touched it.
    pub was_autohide: bool,
    /// Whether the "show hidden icons" chevron existed while the taskbar was still visible.
    pub chevron_present: bool,
    pub app_pid: u32,
}

/// None for empty, corrupt or unknown-version input.
pub fn state_from_json(text: &str) -> Option<TaskbarState> {
    let state: TaskbarState = serde_json::from_str(text).ok()?;
    (state.version == STATE_VERSION).then_some(state)
}

pub fn state_to_json(state: &TaskbarState) -> String {
    serde_json::to_string_pretty(state).unwrap_or_default()
}

#[derive(Debug, Clone, PartialEq)]
pub struct HidePlan {
    pub state_to_save: TaskbarState,
    pub set_autohide: bool,
    pub hide_window: bool,
}

/// While a state hidden by MacDows already exists (crash and restart), its original values are kept: the
/// current auto-hide value is our own change then, not the user's.
pub fn plan_hide(existing: Option<&TaskbarState>, current_autohide: bool, chevron_present: bool, pid: u32) -> HidePlan {
    let (was_autohide, chevron) = match existing {
        Some(s) if s.hidden_by_macdows => (s.was_autohide, s.chevron_present),
        _ => (current_autohide, chevron_present),
    };
    HidePlan {
        state_to_save: TaskbarState { version: STATE_VERSION, hidden_by_macdows: true, was_autohide, chevron_present: chevron, app_pid: pid },
        set_autohide: true,
        hide_window: true,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RestorePlan {
    pub show_window: bool,
    pub set_autohide: Option<bool>,
    pub clear_state: bool,
}

/// Without a state the auto-hide value is never guessed; the window is only shown when it is hidden.
pub fn plan_restore(state: Option<&TaskbarState>, window_hidden_now: bool) -> RestorePlan {
    match state {
        Some(s) if s.hidden_by_macdows => RestorePlan { show_window: true, set_autohide: Some(s.was_autohide), clear_state: true },
        Some(_) => RestorePlan { show_window: false, set_autohide: None, clear_state: false },
        None => RestorePlan { show_window: window_hidden_now, set_autohide: None, clear_state: false },
    }
}

pub fn guard_should_restore(parent_alive: bool, state: Option<&TaskbarState>) -> bool {
    !parent_alive && state.map_or(false, |s| s.hidden_by_macdows)
}

/// Win+B then Enter presses the first tray item; it is the chevron only if the chevron existed.
pub fn hidden_icons_available(state: &TaskbarState, taskbar_hidden: bool) -> bool {
    state.chevron_present && taskbar_hidden
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAction {
    Notifications,
    QuickSettings,
    HiddenIcons,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStep {
    Chord { win: bool, key: u8 },
    Wait { ms: u32 },
}

/// Esc is never sent: it would reach whichever window has focus.
pub fn key_steps(action: ShellAction) -> Vec<KeyStep> {
    match action {
        ShellAction::Notifications => vec![KeyStep::Chord { win: true, key: 0x4E }],
        ShellAction::QuickSettings => vec![KeyStep::Chord { win: true, key: 0x41 }],
        ShellAction::HiddenIcons => vec![KeyStep::Chord { win: true, key: 0x42 }, KeyStep::Wait { ms: 700 }, KeyStep::Chord { win: false, key: 0x0D }],
    }
}

/// Value of the autostart registry entry: the exe path in double quotes.
pub fn run_value(exe: &str) -> String {
    format!("\"{exe}\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hidden(was_autohide: bool) -> TaskbarState {
        TaskbarState { version: 1, hidden_by_macdows: true, was_autohide, chevron_present: true, app_pid: 7 }
    }

    #[test]
    fn state_json_roundtrips_and_rejects_bad_input() {
        let s = hidden(true);
        assert_eq!(state_from_json(&state_to_json(&s)), Some(s));
        assert_eq!(state_from_json(""), None);
        assert_eq!(state_from_json("{not json"), None);
        assert_eq!(state_from_json(r#"{"version":2,"hidden_by_macdows":true,"was_autohide":false,"chevron_present":true,"app_pid":1}"#), None);
    }

    #[test]
    fn hiding_records_the_real_original() {
        let off = plan_hide(None, false, true, 7);
        assert_eq!(off.state_to_save, hidden(false));
        assert!(off.set_autohide && off.hide_window);
        assert!(plan_hide(None, true, true, 7).state_to_save.was_autohide);
    }

    #[test]
    fn a_restart_after_a_crash_keeps_the_stored_original() {
        // The taskbar is already auto-hide (our own change) when the app restarts; the stored original wins.
        let existing = hidden(false);
        let plan = plan_hide(Some(&existing), true, false, 99);
        assert!(!plan.state_to_save.was_autohide);
        assert!(plan.state_to_save.chevron_present, "the chevron seen before the first hide is kept too");
        assert_eq!(plan.state_to_save.app_pid, 99);
    }

    #[test]
    fn restore_puts_back_each_original() {
        let off = plan_restore(Some(&hidden(false)), true);
        assert_eq!(off, RestorePlan { show_window: true, set_autohide: Some(false), clear_state: true });
        let on = plan_restore(Some(&hidden(true)), true);
        assert_eq!(on, RestorePlan { show_window: true, set_autohide: Some(true), clear_state: true });
    }

    #[test]
    fn restore_without_state_never_guesses_auto_hide() {
        assert_eq!(plan_restore(None, true), RestorePlan { show_window: true, set_autohide: None, clear_state: false });
        assert_eq!(plan_restore(None, false), RestorePlan { show_window: false, set_autohide: None, clear_state: false });
        let mut not_ours = hidden(false);
        not_ours.hidden_by_macdows = false;
        assert_eq!(plan_restore(Some(&not_ours), true), RestorePlan { show_window: false, set_autohide: None, clear_state: false });
    }

    #[test]
    fn the_guard_restores_only_after_the_parent_is_gone_and_we_hid_the_taskbar() {
        assert!(!guard_should_restore(true, Some(&hidden(false))));
        assert!(guard_should_restore(false, Some(&hidden(false))));
        assert!(!guard_should_restore(false, None));
    }

    #[test]
    fn hidden_icons_need_the_chevron_and_a_hidden_taskbar() {
        let with = hidden(false);
        let mut without = hidden(false);
        without.chevron_present = false;
        assert!(hidden_icons_available(&with, true));
        assert!(!hidden_icons_available(&without, true));
        assert!(!hidden_icons_available(&with, false));
    }

    #[test]
    fn shell_buttons_send_the_documented_shortcuts() {
        assert_eq!(key_steps(ShellAction::Notifications), vec![KeyStep::Chord { win: true, key: 0x4E }]);
        assert_eq!(key_steps(ShellAction::QuickSettings), vec![KeyStep::Chord { win: true, key: 0x41 }]);
        assert_eq!(
            key_steps(ShellAction::HiddenIcons),
            vec![KeyStep::Chord { win: true, key: 0x42 }, KeyStep::Wait { ms: 700 }, KeyStep::Chord { win: false, key: 0x0D }]
        );
    }

    #[test]
    fn the_autostart_value_is_the_quoted_path() {
        assert_eq!(run_value(r"C:\A B\macdows.exe"), r#""C:\A B\macdows.exe""#);
    }
}
