use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Fixed,
    Smart,
    Peek,
}

/// A pinned app that only MacDows knows: Windows would not pin it to its own taskbar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LocalPin {
    pub id: String,
    pub name: String,
    pub exe: String,
    /// AppUserModelID of a Store app ("" otherwise).
    pub aumid: String,
}

/// What MacDows adds on top of the pinned apps Windows lists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Dock {
    /// Pin ids in the order the user dragged them; pins not listed go last, listed ones that are gone are ignored.
    pub pinned_order: Vec<String>,
    pub local_pins: Vec<LocalPin>,
    /// Pin id (or exe path) to the file name of its custom icon in the icons folder.
    pub custom_icons: HashMap<String, String>,
    pub show_drives: bool,
    /// Window previews on hover.
    pub previews: bool,
    /// Win+1..9 start or focus the pinned apps.
    pub win_number: bool,
}

impl Default for Dock {
    fn default() -> Self {
        Dock { pinned_order: Vec::new(), local_pins: Vec::new(), custom_icons: HashMap::new(), show_drives: true, previews: true, win_number: true }
    }
}

/// What the user chose for one monitor (found by `key`, see `monitors::monitor_key`): whether it has an island and a dock.
/// `name` is only the last display name seen, so a monitor that is not plugged in can still be shown in Settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonitorCfg {
    pub key: String,
    pub name: String,
    pub island: bool,
    pub dock: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub dock_visibility: Visibility,
    /// Look for new releases by itself (the Check now button always works).
    pub check_updates: bool,
    /// Unix seconds of the last finished check.
    pub last_update_check: Option<u64>,
    /// Hide the Windows taskbar and take its place.
    pub replace_taskbar: bool,
    /// The one-time explanation before the first hide was shown.
    pub taskbar_notice_shown: bool,
    /// Start MacDows when Windows starts.
    pub start_with_windows: bool,
    /// A dock button that jumps to the desktop.
    pub show_desktop_button: bool,
    /// Show the apps pinned to the Windows taskbar in the dock.
    pub show_pinned_apps: bool,
    /// Pins removed in the dock that Windows would not unpin (compared ignoring case); the Windows taskbar keeps them.
    pub hidden_pinned: Vec<String>,
    pub dock: Dock,
    /// The Dynamic Island: its screens, what they offer, how it is navigated.
    pub island: crate::island_settings::Island,
    /// The version that ran last; tells the next start whether an update happened.
    pub last_run_version: Option<String>,
    /// Which monitors have an island and a dock. Empty: only the primary monitor has both.
    pub monitors: Vec<MonitorCfg>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "en".into(),
            dock_visibility: Visibility::Fixed,
            check_updates: true,
            last_update_check: None,
            replace_taskbar: true,
            taskbar_notice_shown: false,
            start_with_windows: false,
            show_desktop_button: false,
            show_pinned_apps: true,
            hidden_pinned: Vec::new(),
            dock: Dock::default(),
            island: crate::island_settings::Island::default(),
            last_run_version: None,
            monitors: Vec::new(),
        }
    }
}

impl Settings {
    /// Never fails: empty or corrupt input yields defaults.
    pub fn from_json(s: &str) -> Settings {
        let Ok(mut value) = serde_json::from_str::<serde_json::Value>(s) else { return Settings::default() };
        migrate(&mut value);
        let mut settings = match serde_json::from_value::<Settings>(value.clone()) {
            Ok(settings) => settings,
            Err(_) => {
                // One bad value in the island settings must not reset everything else.
                if let Some(object) = value.as_object_mut() {
                    object.remove("island");
                }
                serde_json::from_value(value).unwrap_or_default()
            }
        };
        settings.island.normalize();
        settings.normalize_monitors();
        settings
    }

    /// A monitor appears once in the list; when it was saved twice the first entry stays.
    pub fn normalize_monitors(&mut self) {
        let mut seen = std::collections::HashSet::new();
        self.monitors.retain(|m| seen.insert(m.key.clone()));
    }
}

/// Moves old settings to where they live now: the island's visibility used to be `notch_visibility`.
fn migrate(value: &mut serde_json::Value) {
    let Some(object) = value.as_object_mut() else { return };
    if !object.contains_key("island") {
        if let Some(visibility) = object.get("notch_visibility").cloned() {
            object.insert("island".into(), serde_json::json!({ "visibility": visibility }));
        }
    }
}

fn path() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("MacDows").join("settings.json")
}

/// Set when this run found no settings file at all: the very first start after an install.
static FIRST_RUN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// True when MacDows is starting for the first time (no settings file existed). Stays the same for the whole run.
pub fn first_run() -> bool {
    FIRST_RUN.load(std::sync::atomic::Ordering::Relaxed)
}

/// A file that is not there at all (not a read error such as a locked or unreadable file) means a first start.
fn is_missing(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::NotFound
}

pub fn load() -> Settings {
    let p = path();
    let text = match std::fs::read_to_string(&p) {
        Ok(text) => text,
        Err(e) => {
            if is_missing(&e) {
                FIRST_RUN.store(true, std::sync::atomic::Ordering::Relaxed);
                // Written right away, so the next start is not a first start.
                let s = Settings::default();
                let _ = save(&s);
                return s;
            }
            return Settings::default();
        }
    };
    if text.trim().is_empty() {
        return Settings::default();
    }
    if serde_json::from_str::<Settings>(&text).is_err() {
        let _ = std::fs::rename(&p, p.with_extension("json.bad"));
        let s = Settings::default();
        let _ = save(&s);
        return s;
    }
    Settings::from_json(&text)
}

pub fn save(s: &Settings) -> std::io::Result<()> {
    let p = path();
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(p, serde_json::to_string_pretty(s).map_err(std::io::Error::other)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_missing_file_counts_as_a_first_start() {
        assert!(is_missing(&std::io::Error::from(std::io::ErrorKind::NotFound)));
        assert!(!is_missing(&std::io::Error::from(std::io::ErrorKind::PermissionDenied)));
        assert!(!is_missing(&std::io::Error::from(std::io::ErrorKind::InvalidData)));
    }

    #[test]
    fn empty_string_gives_defaults() {
        let s = Settings::from_json("");
        assert_eq!(s.language, "en");
        assert_eq!(s.island.visibility, Visibility::Fixed);
        assert_eq!(s.dock_visibility, Visibility::Fixed);
    }

    #[test]
    fn corrupt_json_gives_defaults() {
        assert_eq!(Settings::from_json("{not json"), Settings::default());
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let s = Settings::from_json(r#"{"language":"tr","future_key":42}"#);
        assert_eq!(s.language, "tr");
    }

    fn cfg(key: &str, island: bool, dock: bool) -> MonitorCfg {
        MonitorCfg { key: key.into(), name: format!("name of {key}"), island, dock }
    }

    #[test]
    fn an_old_settings_file_has_no_monitors() {
        assert!(Settings::from_json(r#"{"language":"tr"}"#).monitors.is_empty());
        assert!(Settings::default().monitors.is_empty());
    }

    #[test]
    fn the_same_monitor_is_kept_once() {
        let mut s = Settings::default();
        s.monitors = vec![cfg("a", true, true), cfg("a", false, false), cfg("b", false, true)];
        s.normalize_monitors();
        assert_eq!(s.monitors, vec![cfg("a", true, true), cfg("b", false, true)]);
    }

    #[test]
    fn a_saved_monitor_list_is_cleaned_when_loaded() {
        let s = Settings::from_json(r#"{"monitors":[{"key":"a","name":"A","island":true,"dock":true},{"key":"a","name":"A2","island":false,"dock":false}]}"#);
        assert_eq!(s.monitors.len(), 1);
        assert_eq!(s.monitors[0].name, "A");
    }

    #[test]
    fn roundtrip_keeps_values() {
        let s = Settings {
            language: "tr".into(),
            island: crate::island_settings::Island { visibility: Visibility::Peek, ..Default::default() },
            dock_visibility: Visibility::Fixed,
            check_updates: false,
            last_update_check: Some(1_700_000_000),
            replace_taskbar: false,
            taskbar_notice_shown: true,
            start_with_windows: false,
            show_desktop_button: true,
            show_pinned_apps: false,
            hidden_pinned: vec!["a".into()],
            dock: Dock { win_number: false, ..Dock::default() },
            last_run_version: Some("0.2.2".into()),
            monitors: vec![cfg("monitor\\del4012", true, false)],
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(Settings::from_json(&json), s);
    }

    #[test]
    fn update_fields_default_to_checking_and_never_checked() {
        let s = Settings::default();
        assert!(s.check_updates);
        assert_eq!(s.last_update_check, None);
    }

    #[test]
    fn taskbar_fields_default_to_replace_notice_pending_and_autostart() {
        let s = Settings::default();
        assert!(s.replace_taskbar);
        assert!(!s.taskbar_notice_shown);
        assert!(!s.start_with_windows);
        assert!(!s.show_desktop_button);
        let old = Settings::from_json(r#"{"language":"tr"}"#);
        assert!(old.replace_taskbar && !old.taskbar_notice_shown && !old.start_with_windows);
    }

    #[test]
    fn a_v010_settings_file_without_update_fields_still_loads() {
        let s = Settings::from_json(r#"{"language":"tr","notch_visibility":"peek","dock_visibility":"fixed"}"#);
        assert_eq!(s.language, "tr");
        assert_eq!(s.island.visibility, Visibility::Peek);
        assert!(s.check_updates);
        assert_eq!(s.last_update_check, None);
    }

    #[test]
    fn dock_defaults_match_the_spec() {
        let d = Settings::default().dock;
        assert!(d.pinned_order.is_empty());
        assert!(d.local_pins.is_empty());
        assert!(d.custom_icons.is_empty());
        assert!(d.show_drives && d.previews && d.win_number);
    }

    #[test]
    fn a_partial_dock_object_is_filled_with_defaults() {
        let s = Settings::from_json(r#"{"dock":{"previews":false,"pinned_order":["a"]}}"#);
        assert!(!s.dock.previews);
        assert_eq!(s.dock.pinned_order, vec!["a".to_string()]);
        assert!(s.dock.show_drives && s.dock.win_number);
    }

    #[test]
    fn an_old_settings_file_without_dock_loads() {
        let s = Settings::from_json(r#"{"language":"tr"}"#);
        assert_eq!(s.language, "tr");
        assert_eq!(s.dock, Dock::default());
    }

    #[test]
    fn a_local_pin_round_trips() {
        let mut s = Settings::default();
        s.dock.local_pins.push(LocalPin { id: "local:calc".into(), name: "Calculator".into(), exe: "C:/calc.exe".into(), aumid: String::new() });
        s.dock.custom_icons.insert("x".into(), "abc.png".into());
        let back = Settings::from_json(&serde_json::to_string(&s).unwrap());
        assert_eq!(back.dock, s.dock);
    }

    #[test]
    fn old_notch_visibility_moves_into_the_island() {
        let s = Settings::from_json(r#"{"language":"tr","notch_visibility":"peek"}"#);
        assert_eq!(s.island.visibility, Visibility::Peek);
        assert_eq!(s.language, "tr");
    }

    #[test]
    fn an_existing_island_object_wins_over_the_old_key() {
        let s = Settings::from_json(r#"{"notch_visibility":"peek","island":{"visibility":"smart"}}"#);
        assert_eq!(s.island.visibility, Visibility::Smart);
    }

    #[test]
    fn an_old_file_without_island_gets_every_island_default() {
        let s = Settings::from_json(r#"{"language":"tr"}"#);
        assert_eq!(s.island, crate::island_settings::Island::default());
    }

    #[test]
    fn loading_repairs_the_island_values() {
        let s = Settings::from_json(r#"{"island":{"nav":{"order":["shelf"]},"screens":{"notifications":{"banner_seconds":99}}}}"#);
        assert_eq!(s.island.nav.order.len(), 6);
        assert_eq!(s.island.nav.order[0], "shelf");
        assert_eq!(s.island.screens.notifications.banner_seconds, 10);
    }

    #[test]
    fn a_bad_island_value_does_not_reset_the_other_settings() {
        let s = Settings::from_json(r#"{"language":"tr","show_desktop_button":true,"island":{"visibility":"bogus"}}"#);
        assert_eq!(s.language, "tr");
        assert!(s.show_desktop_button);
        assert_eq!(s.island, crate::island_settings::Island::default());
    }
}
