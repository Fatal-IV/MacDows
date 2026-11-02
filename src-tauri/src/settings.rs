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
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: "en".into(),
            notch_visibility: Visibility::Smart,
            dock_visibility: Visibility::Smart,
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
        assert_eq!(s.notch_visibility, Visibility::Smart);
        assert_eq!(s.dock_visibility, Visibility::Smart);
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
        let s = Settings { language: "tr".into(), notch_visibility: Visibility::Peek, dock_visibility: Visibility::Fixed };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(Settings::from_json(&json), s);
    }
}
