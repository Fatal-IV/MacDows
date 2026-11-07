//! Monitors and the bars on them. This file starts with the pure part (which bar windows should exist for the monitors and the
//! settings); what creates, moves and removes the windows comes after it.

use crate::bar::{self, BarKind};
use crate::fullscreen::Rect;
use crate::settings::MonitorCfg;

/// One monitor as Windows reports it (physical pixels).
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
    pub key: String,
    pub name: String,
    pub rect: Rect,
    /// The part of the monitor not taken by a taskbar.
    pub work: Rect,
    pub scale: f64,
    pub primary: bool,
}

/// What identifies a monitor in the settings: the hardware device ID (lowercased) because `\.\DISPLAYn` can be renumbered by Windows;
/// the display name only when no device ID can be read.
pub fn monitor_key(device_id: Option<&str>, name: &str) -> String {
    match device_id.map(str::trim).filter(|id| !id.is_empty()) {
        Some(id) => id.to_lowercase(),
        None => name.to_string(),
    }
}

/// Whether a monitor should have an island and a dock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Want {
    pub key: String,
    pub island: bool,
    pub dock: bool,
}

/// One entry per monitor that is plugged in: what the settings say, or (no entry yet) both bars on the primary monitor and none elsewhere.
pub fn desired(monitors: &[MonitorInfo], cfg: &[MonitorCfg]) -> Vec<Want> {
    monitors
        .iter()
        .map(|m| match cfg.iter().find(|c| c.key == m.key) {
            Some(c) => Want { key: m.key.clone(), island: c.island, dock: c.dock },
            None => Want { key: m.key.clone(), island: m.primary, dock: m.primary },
        })
        .collect()
}

pub fn is_wanted(want: &[Want], key: &str, kind: BarKind) -> bool {
    want.iter().find(|w| w.key == key).is_some_and(|w| match kind {
        BarKind::Notch => w.island,
        BarKind::Dock => w.dock,
    })
}

/// A bar window that exists, and the monitor it is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BarWin {
    pub label: String,
    pub kind: BarKind,
    pub key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Create { kind: BarKind, key: String },
    Destroy { label: String },
}

/// What to do so that the bar windows match the wishes. The plain `notch` and `dock` windows belong to the primary monitor, always exist
/// and are never in the result (a switched-off one is hidden by the visibility loop); numbered ones are created and destroyed.
/// Destroys come first.
pub fn plan(monitors: &[MonitorInfo], want: &[Want], existing: &[BarWin]) -> Vec<Action> {
    let mut actions = Vec::new();
    let numbered = |w: &&BarWin| matches!(bar::parse(&w.label), Some((_, Some(_))));
    for w in existing.iter().filter(numbered) {
        let monitor = monitors.iter().find(|m| m.key == w.key);
        let keep = monitor.is_some_and(|m| !m.primary) && is_wanted(want, &w.key, w.kind);
        if !keep {
            actions.push(Action::Destroy { label: w.label.clone() });
        }
    }
    for m in monitors.iter().filter(|m| !m.primary) {
        for kind in [BarKind::Notch, BarKind::Dock] {
            let has = existing.iter().filter(numbered).any(|w| w.key == m.key && w.kind == kind);
            if is_wanted(want, &m.key, kind) && !has {
                actions.push(Action::Create { kind, key: m.key.clone() });
            }
        }
    }
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(left: i32, right: i32) -> Rect {
        Rect { left, top: 0, right, bottom: 1080 }
    }

    fn mon(key: &str, primary: bool, x: i32) -> MonitorInfo {
        MonitorInfo { key: key.into(), name: format!("name of {key}"), rect: r(x, x + 1920), work: r(x, x + 1920), scale: 1.0, primary }
    }

    fn cfg(key: &str, island: bool, dock: bool) -> MonitorCfg {
        MonitorCfg { key: key.into(), name: String::new(), island, dock }
    }

    fn want(key: &str, island: bool, dock: bool) -> Want {
        Want { key: key.into(), island, dock }
    }

    fn win(label: &str, key: &str) -> BarWin {
        let (kind, _) = bar::parse(label).unwrap();
        BarWin { label: label.into(), kind, key: key.into() }
    }

    #[test]
    fn the_key_prefers_the_device_id() {
        assert_eq!(monitor_key(Some("MONITOR\\DEL4012"), "\\\\.\\DISPLAY2"), "monitor\\del4012");
        assert_eq!(monitor_key(None, "\\\\.\\DISPLAY2"), "\\\\.\\DISPLAY2");
        assert_eq!(monitor_key(Some(""), "\\\\.\\DISPLAY2"), "\\\\.\\DISPLAY2");
        assert_eq!(monitor_key(Some("   "), "\\\\.\\DISPLAY2"), "\\\\.\\DISPLAY2");
    }

    #[test]
    fn by_default_only_the_primary_monitor_has_bars() {
        let w = desired(&[mon("a", true, 0), mon("b", false, 1920)], &[]);
        assert_eq!(w, vec![want("a", true, true), want("b", false, false)]);
    }

    #[test]
    fn a_saved_entry_wins() {
        let w = desired(&[mon("a", true, 0), mon("b", false, 1920)], &[cfg("a", false, true), cfg("b", true, true)]);
        assert_eq!(w, vec![want("a", false, true), want("b", true, true)]);
    }

    #[test]
    fn a_saved_monitor_that_is_not_plugged_in_is_ignored() {
        let w = desired(&[mon("a", true, 0)], &[cfg("gone", true, true)]);
        assert_eq!(w, vec![want("a", true, true)]);
    }

    #[test]
    fn a_wanted_second_monitor_gets_both_bars() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let actions = plan(&ms, &[want("a", true, true), want("b", true, true)], &[win("notch", "a"), win("dock", "a")]);
        assert_eq!(actions, vec![Action::Create { kind: BarKind::Notch, key: "b".into() }, Action::Create { kind: BarKind::Dock, key: "b".into() }]);
    }

    #[test]
    fn only_the_wanted_bar_is_created() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let actions = plan(&ms, &[want("a", true, true), want("b", false, true)], &[]);
        assert_eq!(actions, vec![Action::Create { kind: BarKind::Dock, key: "b".into() }]);
    }

    #[test]
    fn nothing_to_do_when_the_windows_exist() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let existing = [win("notch", "a"), win("dock", "a"), win("notch-2", "b"), win("dock-2", "b")];
        assert!(plan(&ms, &[want("a", true, true), want("b", true, true)], &existing).is_empty());
    }

    #[test]
    fn a_bar_that_is_switched_off_is_destroyed() {
        let ms = [mon("a", true, 0), mon("b", false, 1920)];
        let existing = [win("notch-2", "b"), win("dock-2", "b")];
        let actions = plan(&ms, &[want("a", true, true), want("b", true, false)], &existing);
        assert_eq!(actions, vec![Action::Destroy { label: "dock-2".into() }]);
    }

    #[test]
    fn an_unplugged_monitor_destroys_only_its_windows() {
        let ms = [mon("a", true, 0), mon("c", false, 3840)];
        let existing = [win("dock-2", "b"), win("dock-3", "c")];
        let actions = plan(&ms, &[want("a", true, true), want("c", false, true)], &existing);
        assert_eq!(actions, vec![Action::Destroy { label: "dock-2".into() }]);
    }

    #[test]
    fn a_new_primary_monitor_loses_its_numbered_pair_and_the_old_primary_gets_one() {
        // b was a second monitor with notch-2 and dock-2; now b is the primary and a is the second one.
        let ms = [mon("a", false, 0), mon("b", true, 1920)];
        let existing = [win("notch", "a"), win("dock", "a"), win("notch-2", "b"), win("dock-2", "b")];
        let actions = plan(&ms, &[want("a", true, true), want("b", true, true)], &existing);
        assert_eq!(
            actions,
            vec![
                Action::Destroy { label: "notch-2".into() },
                Action::Destroy { label: "dock-2".into() },
                Action::Create { kind: BarKind::Notch, key: "a".into() },
                Action::Create { kind: BarKind::Dock, key: "a".into() },
            ]
        );
    }

    #[test]
    fn the_plain_windows_are_never_touched() {
        let ms = [mon("a", true, 0)];
        let actions = plan(&ms, &[want("a", false, false)], &[win("notch", "a"), win("dock", "a")]);
        assert!(actions.is_empty(), "a switched-off primary bar is hidden by the visibility loop, not closed");
    }

    #[test]
    fn is_wanted_reads_the_right_flag() {
        let w = [want("a", true, false)];
        assert!(is_wanted(&w, "a", BarKind::Notch));
        assert!(!is_wanted(&w, "a", BarKind::Dock));
        assert!(!is_wanted(&w, "missing", BarKind::Notch));
    }
}
