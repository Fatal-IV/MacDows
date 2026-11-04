use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Fixed,
    Smart,
    Peek,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub notch_visibility: Visibility,
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
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "en".into(),
            notch_visibility: Visibility::Fixed,
            dock_visibility: Visibility::Fixed,
            check_updates: true,
            last_update_check: None,
            replace_taskbar: true,
            taskbar_notice_shown: false,
            start_with_windows: false,
            show_desktop_button: false,
            show_pinned_apps: true,
            hidden_pinned: Vec::new(),
        }
    }
}

impl Settings {
    /// Never fails: empty or corrupt input yields defaults.
    pub fn from_json(s: &str) -> Settings {
        serde_json::from_str(s).unwrap_or_default()
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
        assert_eq!(s.notch_visibility, Visibility::Fixed);
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
            notch_visibility: Visibility::Peek,
            dock_visibility: Visibility::Fixed,
            check_updates: false,
            last_update_check: Some(1_700_000_000),
            replace_taskbar: false,
            taskbar_notice_shown: true,
            start_with_windows: false,
            show_desktop_button: true,
            show_pinned_apps: false,
            hidden_pinned: vec!["a".into()],
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
        assert_eq!(s.notch_visibility, Visibility::Peek);
        assert!(s.check_updates);
        assert_eq!(s.last_update_check, None);
    }
}
