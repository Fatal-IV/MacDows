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

/// After a restore (or when the owner is gone) a hidden taskbar window is a bug: nobody owns that state any more.
pub fn needs_final_show(state: Option<&TaskbarState>, window_hidden: bool) -> bool {
    state.is_none() && window_hidden
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
    Start,
    Search,
    ShowDesktop,
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
        // A bare Windows key press opens Start; Win+S opens Search.
        ShellAction::Start => vec![KeyStep::Chord { win: false, key: 0x5B }],
        ShellAction::Search => vec![KeyStep::Chord { win: true, key: 0x53 }],
        // Win+D shows the desktop and, pressed again, brings the windows back.
        ShellAction::ShowDesktop => vec![KeyStep::Chord { win: true, key: 0x44 }],
        ShellAction::HiddenIcons => {
            // Win+B focuses the last focused tray item, not necessarily the "show hidden icons" chevron. The chevron is
            // always the leftmost item and Home jumps straight to it. Walking Left does not work: focus wraps around, so
            // with fewer than 16 items it can land on the clock and Enter opens the notification center instead.
            vec![
                KeyStep::Chord { win: true, key: 0x42 },
                KeyStep::Wait { ms: 700 },
                KeyStep::Chord { win: false, key: 0x24 },
                KeyStep::Wait { ms: 300 },
                KeyStep::Chord { win: false, key: 0x0D },
            ]
        }
    }
}

/// Explorer can show the taskbar again after we hid it (quick changes, restarts). While the setting is on and the
/// state says we hid it, a visible taskbar is hidden again.
pub fn watchdog_should_rehide(setting_on: bool, state: Option<&TaskbarState>, window_visible: bool) -> bool {
    setting_on && window_visible && state.map_or(false, |s| s.hidden_by_macdows)
}

/// Value of the autostart registry entry: the exe path in double quotes.
pub fn run_value(exe: &str) -> String {
    format!("\"{exe}\"")
}

/// A guard may only undo the state that its own app wrote, so an old guard never undoes a newer instance.
pub fn state_belongs_to(state: &TaskbarState, pid: u32) -> bool {
    state.app_pid == pid
}

/// The guard is a copy of macdows.exe under another name (the installer ends macdows.exe by name on
/// update and uninstall). It is copied again only when missing or of a different size.
pub fn guard_needs_refresh(source_len: u64, copy_len: Option<u64>) -> bool {
    copy_len != Some(source_len)
}

/// At every start: a state that says "hidden by MacDows" while its app is gone means a crash happened and the
/// guard could not restore; restore before anything else.
pub fn needs_stale_restore(state: Option<&TaskbarState>, app_alive: bool) -> bool {
    state.map_or(false, |s| s.hidden_by_macdows) && !app_alive
}

/// Release builds follow the setting. A debug build (developer running `tauri dev`) only touches the
/// taskbar when `MACDOWS_REPLACE_TASKBAR` is set, so editing the code never hides the developer's taskbar.
pub fn should_replace_now(setting: bool, debug_build: bool, env_opt_in: bool) -> bool {
    setting && (!debug_build || env_opt_in)
}

/// The Windows side of the taskbar. The real implementation is `taskbar::WinBackend`.
pub trait Backend {
    fn autohide(&self) -> bool;
    fn set_autohide(&self, on: bool);
    fn hide_window(&self);
    fn show_window(&self);
    fn window_hidden(&self) -> bool;
    /// Must be read while the taskbar is still visible.
    fn chevron_present(&self) -> bool;
}

/// Where the state lives. The real implementation is `taskbar::FileStore`.
pub trait Store {
    fn load(&self) -> Option<TaskbarState>;
    fn save(&self, state: &TaskbarState) -> std::io::Result<()>;
    fn clear(&self);
}

#[derive(Debug, PartialEq, Eq)]
pub enum HideError {
    StateNotSaved,
}

/// Saves the state first; if that fails nothing is touched. Order: auto-hide, then the window.
pub fn hide(backend: &impl Backend, store: &impl Store, pid: u32) -> Result<(), HideError> {
    let existing = store.load();
    let plan = plan_hide(existing.as_ref(), backend.autohide(), backend.chevron_present(), pid);
    store.save(&plan.state_to_save).map_err(|_| HideError::StateNotSaved)?;
    if plan.set_autohide {
        backend.set_autohide(true);
    }
    if plan.hide_window {
        backend.hide_window();
    }
    Ok(())
}

/// Order: show the window, then put the stored auto-hide value back.
pub fn restore(backend: &impl Backend, store: &impl Store) {
    let state = store.load();
    let plan = plan_restore(state.as_ref(), backend.window_hidden());
    if plan.show_window {
        backend.show_window();
    }
    if let Some(on) = plan.set_autohide {
        backend.set_autohide(on);
    }
    // Changing auto-hide makes Explorer re-lay the taskbar out, which can hide the window again.
    if plan.show_window && backend.window_hidden() {
        backend.show_window();
    }
    if plan.clear_state {
        store.clear();
    }
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
    }

    #[test]
    fn the_desktop_button_sends_win_d() {
        assert_eq!(key_steps(ShellAction::ShowDesktop), vec![KeyStep::Chord { win: true, key: 0x44 }]);
    }

    #[test]
    fn start_is_a_bare_windows_key_press_and_search_is_win_s() {
        assert_eq!(key_steps(ShellAction::Start), vec![KeyStep::Chord { win: false, key: 0x5B }]);
        assert_eq!(key_steps(ShellAction::Search), vec![KeyStep::Chord { win: true, key: 0x53 }]);
    }

    #[test]
    fn hidden_icons_jump_to_the_chevron_with_home_before_pressing_enter() {
        // Win+B focuses the LAST focused tray item, not the chevron. Home reaches the leftmost item (the chevron); Left
        // presses wrap around and can end on the clock, which opens the notification center.
        let steps = key_steps(ShellAction::HiddenIcons);
        assert_eq!(steps[0], KeyStep::Chord { win: true, key: 0x42 });
        assert_eq!(steps[1], KeyStep::Wait { ms: 700 });
        assert!(!steps.iter().any(|s| matches!(s, KeyStep::Chord { key: 0x25, .. })), "Left wraps around and must not be used");
        assert!(!steps.iter().any(|s| matches!(s, KeyStep::Chord { key: 0x1B, .. })), "Esc must never be sent");
        let home_at = steps.iter().position(|s| matches!(s, KeyStep::Chord { win: false, key: 0x24 })).unwrap();
        let enter_at = steps.iter().position(|s| matches!(s, KeyStep::Chord { win: false, key: 0x0D })).unwrap();
        assert!(home_at < enter_at, "Home comes before Enter");
        assert_eq!(*steps.last().unwrap(), KeyStep::Chord { win: false, key: 0x0D });
    }

    #[test]
    fn the_autostart_value_is_the_quoted_path() {
        assert_eq!(run_value(r"C:\A B\macdows.exe"), r#""C:\A B\macdows.exe""#);
    }
}

#[cfg(test)]
mod orchestration_tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    type Log = Rc<RefCell<Vec<String>>>;

    struct FakeBackend {
        autohide: Cell<bool>,
        hidden: Cell<bool>,
        chevron: bool,
        log: Log,
    }

    impl Backend for FakeBackend {
        fn autohide(&self) -> bool {
            self.autohide.get()
        }
        fn set_autohide(&self, on: bool) {
            self.log.borrow_mut().push(format!("set_autohide({on})"));
            self.autohide.set(on);
        }
        fn hide_window(&self) {
            self.log.borrow_mut().push("hide_window".into());
            self.hidden.set(true);
        }
        fn show_window(&self) {
            self.log.borrow_mut().push("show_window".into());
            self.hidden.set(false);
        }
        fn window_hidden(&self) -> bool {
            self.hidden.get()
        }
        fn chevron_present(&self) -> bool {
            self.chevron
        }
    }

    struct FakeStore {
        state: RefCell<Option<TaskbarState>>,
        fail_save: bool,
        log: Log,
    }

    impl Store for FakeStore {
        fn load(&self) -> Option<TaskbarState> {
            self.state.borrow().clone()
        }
        fn save(&self, s: &TaskbarState) -> std::io::Result<()> {
            if self.fail_save {
                self.log.borrow_mut().push("save-failed".into());
                return Err(std::io::Error::other("disk full"));
            }
            self.log.borrow_mut().push("save".into());
            *self.state.borrow_mut() = Some(s.clone());
            Ok(())
        }
        fn clear(&self) {
            self.log.borrow_mut().push("clear".into());
            *self.state.borrow_mut() = None;
        }
    }

    fn rig(autohide: bool, fail_save: bool, state: Option<TaskbarState>) -> (FakeBackend, FakeStore, Log) {
        let log: Log = Rc::new(RefCell::new(Vec::new()));
        let backend = FakeBackend { autohide: Cell::new(autohide), hidden: Cell::new(false), chevron: true, log: log.clone() };
        let store = FakeStore { state: RefCell::new(state), fail_save, log: log.clone() };
        (backend, store, log)
    }

    #[test]
    fn the_state_is_saved_before_anything_is_hidden() {
        let (b, s, log) = rig(false, false, None);
        hide(&b, &s, 7).unwrap();
        assert_eq!(*log.borrow(), vec!["save", "set_autohide(true)", "hide_window"]);
    }

    #[test]
    fn nothing_is_hidden_when_the_state_cannot_be_saved() {
        let (b, s, log) = rig(false, true, None);
        assert_eq!(hide(&b, &s, 7), Err(HideError::StateNotSaved));
        assert_eq!(*log.borrow(), vec!["save-failed"]);
        assert!(!b.hidden.get());
    }

    #[test]
    fn hide_then_restore_returns_to_auto_hide_off() {
        let (b, s, _) = rig(false, false, None);
        hide(&b, &s, 7).unwrap();
        assert!(b.hidden.get() && b.autohide.get());
        restore(&b, &s);
        assert!(!b.hidden.get());
        assert!(!b.autohide.get());
        assert_eq!(s.load(), None);
    }

    #[test]
    fn hide_then_restore_returns_to_auto_hide_on_when_it_was_on() {
        let (b, s, _) = rig(true, false, None);
        hide(&b, &s, 7).unwrap();
        restore(&b, &s);
        assert!(!b.hidden.get());
        assert!(b.autohide.get(), "the user had auto-hide on before MacDows");
    }

    #[test]
    fn hiding_again_after_a_crash_keeps_the_first_original() {
        let (b, s, _) = rig(false, false, None);
        hide(&b, &s, 7).unwrap();
        // The app crashed and restarted: the taskbar is already in our modified state.
        hide(&b, &s, 8).unwrap();
        assert!(!s.load().unwrap().was_autohide);
        assert_eq!(s.load().unwrap().app_pid, 8);
        restore(&b, &s);
        assert!(!b.autohide.get());
    }

    #[test]
    fn restore_without_state_shows_the_window_and_leaves_auto_hide_alone() {
        let (b, s, log) = rig(true, false, None);
        b.hidden.set(true);
        restore(&b, &s);
        assert_eq!(*log.borrow(), vec!["show_window"]);
        assert!(b.autohide.get());
    }
}

#[cfg(test)]
mod guard_owner_tests {
    use super::*;

    #[test]
    fn a_guard_only_restores_the_state_its_own_app_wrote() {
        let state = TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: 8 };
        assert!(state_belongs_to(&state, 8));
        assert!(!state_belongs_to(&state, 7), "an old guard must not undo a newer instance");
    }

    #[test]
    fn the_guard_copy_is_refreshed_only_when_missing_or_a_different_size() {
        assert!(guard_needs_refresh(1000, None));
        assert!(guard_needs_refresh(1000, Some(999)));
        assert!(!guard_needs_refresh(1000, Some(1000)));
    }
}

#[cfg(test)]
mod wiring_tests {
    use super::*;

    fn hidden(pid: u32) -> TaskbarState {
        TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: pid }
    }

    #[test]
    fn a_hidden_state_whose_app_is_gone_must_be_restored_at_start() {
        assert!(needs_stale_restore(Some(&hidden(7)), false));
        assert!(!needs_stale_restore(Some(&hidden(7)), true));
        assert!(!needs_stale_restore(None, false));
    }

    #[test]
    fn debug_builds_touch_the_taskbar_only_with_an_explicit_opt_in() {
        assert!(should_replace_now(true, false, false), "release builds follow the setting");
        assert!(!should_replace_now(false, false, true));
        assert!(!should_replace_now(true, true, false), "a developer running `tauri dev` keeps the taskbar");
        assert!(should_replace_now(true, true, true));
        assert!(!should_replace_now(false, true, true));
    }
}

#[cfg(test)]
mod watchdog_tests {
    use super::*;

    fn hidden() -> TaskbarState {
        TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: 7 }
    }

    #[test]
    fn the_watchdog_hides_a_taskbar_that_reappeared_only_while_we_own_the_state() {
        assert!(watchdog_should_rehide(true, Some(&hidden()), true));
        assert!(!watchdog_should_rehide(false, Some(&hidden()), true), "the setting is off");
        assert!(!watchdog_should_rehide(true, None, true), "no state means we are not hiding anything");
        assert!(!watchdog_should_rehide(true, Some(&hidden()), false), "already hidden");
    }
}

#[cfg(test)]
mod final_visibility_tests {
    use super::*;
    use std::cell::Cell;

    /// Explorer re-lays the taskbar out when auto-hide changes and can hide the window again.
    struct MovesTheWindowAway {
        hidden: Cell<bool>,
        autohide: Cell<bool>,
        shows: Cell<u32>,
    }

    impl Backend for MovesTheWindowAway {
        fn autohide(&self) -> bool {
            self.autohide.get()
        }
        fn set_autohide(&self, on: bool) {
            if self.autohide.get() && !on {
                self.hidden.set(true); // the Explorer re-layout hides it
            }
            self.autohide.set(on);
        }
        fn hide_window(&self) {
            self.hidden.set(true);
        }
        fn show_window(&self) {
            self.shows.set(self.shows.get() + 1);
            self.hidden.set(false);
        }
        fn window_hidden(&self) -> bool {
            self.hidden.get()
        }
        fn chevron_present(&self) -> bool {
            true
        }
    }

    struct MemStore(std::cell::RefCell<Option<TaskbarState>>);
    impl Store for MemStore {
        fn load(&self) -> Option<TaskbarState> {
            self.0.borrow().clone()
        }
        fn save(&self, s: &TaskbarState) -> std::io::Result<()> {
            *self.0.borrow_mut() = Some(s.clone());
            Ok(())
        }
        fn clear(&self) {
            *self.0.borrow_mut() = None;
        }
    }

    #[test]
    fn restore_ends_with_a_visible_taskbar_even_if_explorer_hides_it_again() {
        let backend = MovesTheWindowAway { hidden: Cell::new(true), autohide: Cell::new(true), shows: Cell::new(0) };
        let store = MemStore(std::cell::RefCell::new(Some(TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: 7 })));
        restore(&backend, &store);
        assert!(!backend.hidden.get(), "the taskbar must be visible after a restore");
        assert!(!backend.autohide.get());
        assert!(backend.shows.get() >= 2, "it was shown again after the auto-hide change hid it");
    }

    #[test]
    fn an_ownerless_hidden_taskbar_must_be_shown() {
        assert!(needs_final_show(None, true));
        assert!(!needs_final_show(None, false));
        let state = TaskbarState { version: 1, hidden_by_macdows: true, was_autohide: false, chevron_present: true, app_pid: 9 };
        assert!(!needs_final_show(Some(&state), true), "a newer MacDows instance owns it");
    }
}
