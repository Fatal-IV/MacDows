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
        settings
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

pub fn load() -> Settings {
    let p = path();
    let Ok(text) = std::fs::read_to_string(&p) else { return Settings::default() };
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
