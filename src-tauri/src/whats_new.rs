//! "What's new": after an update the Settings window opens with the notes of the new version, in the app's language.
use serde::Serialize;
use std::sync::Mutex;

/// The changelog is built into the program, so the notes are there even when offline.
const CHANGELOG: &str = include_str!("../../CHANGELOG.md");

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WhatsNew {
    pub version: String,
    pub notes: String,
}

#[derive(Default)]
pub struct Pending(pub Mutex<Option<WhatsNew>>);

/// The text of the `## [version]` section, without its heading; None when there is none.
pub fn section(changelog: &str, version: &str) -> Option<String> {
    let lines: Vec<&str> = changelog.lines().collect();
    let start = lines.iter().position(|l| l.starts_with(&format!("## [{version}]")))?;
    let end = lines.iter().skip(start + 1).position(|l| l.starts_with("## [")).map_or(lines.len(), |i| start + 1 + i);
    Some(lines[start + 1..end].join("\n").trim().to_string())
}

/// True on a start that follows an update: settings existed before (not a fresh install) and they were last
/// used by another version (or by a version old enough to not record it).
pub fn is_update(first_run: bool, last_run: Option<&str>, current: &str) -> bool {
    !first_run && last_run != Some(current)
}

/// The notes to show for `version` in `language`, if the changelog has them.
pub fn notes_for(changelog: &str, version: &str, language: &str) -> Option<WhatsNew> {
    let all = section(changelog, version)?;
    let notes = crate::updates::notes_for_language(&all, language);
    (!notes.is_empty()).then(|| WhatsNew { version: version.to_string(), notes })
}

pub fn built_in(version: &str, language: &str) -> Option<WhatsNew> {
    notes_for(CHANGELOG, version, language)
}

/// Hands the notes over once.
#[tauri::command]
pub fn take_whats_new(pending: tauri::State<Pending>) -> Option<WhatsNew> {
    pending.0.lock().unwrap().take()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "# Changelog\n\n## [Unreleased]\n\n## [0.2.3] - 2026-11-07\n\n### Added\n- Eyes\n\n### Türkçe\n### Eklendi\n- Gözler\n\n## [0.2.2] - 2026-11-06\n\n### Fixed\n- Old\n";

    #[test]
    fn a_section_stops_at_the_next_version() {
        let s = section(LOG, "0.2.3").unwrap();
        assert!(s.starts_with("### Added") && s.ends_with("- Gözler"));
        assert!(!s.contains("Old"));
        assert_eq!(section(LOG, "9.9.9"), None);
    }

    #[test]
    fn the_notes_follow_the_language() {
        assert_eq!(notes_for(LOG, "0.2.3", "tr").unwrap().notes, "### Eklendi\n- Gözler");
        assert_eq!(notes_for(LOG, "0.2.3", "en").unwrap().notes, "### Added\n- Eyes");
    }

    #[test]
    fn only_a_start_after_an_update_shows_them() {
        assert!(!is_update(true, None, "0.2.3"), "a fresh install says nothing");
        assert!(!is_update(false, Some("0.2.3"), "0.2.3"), "same version again");
        assert!(is_update(false, Some("0.2.2"), "0.2.3"));
        assert!(is_update(false, None, "0.2.3"), "settings from a version that did not record it");
    }

    #[test]
    fn the_built_in_changelog_has_the_current_version() {
        assert!(built_in(env!("CARGO_PKG_VERSION"), "en").is_some());
    }
}
