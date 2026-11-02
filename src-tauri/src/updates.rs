//! In-app update: pure decisions (unit tested) and the effects on top of tauri-plugin-updater.
use serde::Serialize;

pub const CHECK_INTERVAL_SECS: u64 = 12 * 3600;
pub const FIRST_CHECK_DELAY_SECS: u64 = 30;
pub const NOTES_MAX_CHARS: usize = 600;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate { checked_at: u64 },
    Available { version: String, notes: String },
    Installing { percent: Option<u8> },
    Error { message: String, during: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum CheckOutcome {
    UpToDate,
    Available { version: String, notes: String },
    Failed(String),
}

/// True when a check is due. A last check in the future (clock moved back) counts as due.
pub fn should_check(last: Option<u64>, now: u64, interval_secs: u64) -> bool {
    match last {
        None => true,
        Some(last) if last > now => true,
        Some(last) => now - last >= interval_secs,
    }
}

/// Trims the notes and cuts them to `max_chars` characters (the final ellipsis included), never inside a character.
pub fn shorten_notes(notes: &str, max_chars: usize) -> String {
    let trimmed = notes.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }
    let mut cut: String = trimmed.chars().take(max_chars.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

/// A new check or install may start unless one is already running.
pub fn can_start(status: &UpdateStatus) -> bool {
    !matches!(status, UpdateStatus::Checking | UpdateStatus::Installing { .. })
}

/// Result of a finished check. A failure is only shown when the user started the check; a background
/// failure leaves the status as it was (it is logged elsewhere).
pub fn after_check(prev: &UpdateStatus, outcome: CheckOutcome, user_initiated: bool, now: u64) -> UpdateStatus {
    match outcome {
        CheckOutcome::UpToDate => UpdateStatus::UpToDate { checked_at: now },
        CheckOutcome::Available { version, notes } => UpdateStatus::Available { version, notes: shorten_notes(&notes, NOTES_MAX_CHARS) },
        CheckOutcome::Failed(message) if user_initiated => UpdateStatus::Error { message, during: "check".to_string() },
        CheckOutcome::Failed(_) => prev.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_check_covers_never_recent_due_and_clock_moved_back() {
        let day = CHECK_INTERVAL_SECS;
        let now = 10_000_000;
        assert!(should_check(None, now, day));
        assert!(!should_check(Some(now - 60), now, day));
        assert!(should_check(Some(now - day), now, day));
        assert!(should_check(Some(now - day - 1), now, day));
        assert!(should_check(Some(now + 500), now, day));
    }

    #[test]
    fn short_and_exact_notes_are_unchanged() {
        assert_eq!(shorten_notes("hi", NOTES_MAX_CHARS), "hi");
        let exact = "a".repeat(NOTES_MAX_CHARS);
        assert_eq!(shorten_notes(&exact, NOTES_MAX_CHARS), exact);
        assert_eq!(shorten_notes("  x  ", NOTES_MAX_CHARS), "x");
        assert_eq!(shorten_notes("", NOTES_MAX_CHARS), "");
    }

    #[test]
    fn long_notes_are_cut_to_the_limit_with_an_ellipsis() {
        let cut = shorten_notes(&"a".repeat(700), NOTES_MAX_CHARS);
        assert_eq!(cut.chars().count(), NOTES_MAX_CHARS);
        assert!(cut.ends_with('…'));
    }

    #[test]
    fn multibyte_notes_are_cut_on_a_character_boundary() {
        let text = "ğüşiöçİ".repeat(200);
        let cut = shorten_notes(&text, NOTES_MAX_CHARS);
        assert!(cut.chars().count() <= NOTES_MAX_CHARS);
        assert!(cut.ends_with('…'));
        assert!(text.starts_with(cut.trim_end_matches('…')));
    }

    #[test]
    fn a_second_check_or_install_cannot_start_while_one_runs() {
        assert!(can_start(&UpdateStatus::Idle));
        assert!(can_start(&UpdateStatus::UpToDate { checked_at: 1 }));
        assert!(can_start(&UpdateStatus::Available { version: "0.1.1".into(), notes: String::new() }));
        assert!(can_start(&UpdateStatus::Error { message: "x".into(), during: "check".into() }));
        assert!(!can_start(&UpdateStatus::Checking));
        assert!(!can_start(&UpdateStatus::Installing { percent: Some(10) }));
    }

    #[test]
    fn check_outcomes_map_to_statuses() {
        let prev = UpdateStatus::UpToDate { checked_at: 5 };
        assert_eq!(after_check(&prev, CheckOutcome::UpToDate, false, 99), UpdateStatus::UpToDate { checked_at: 99 });
        assert_eq!(
            after_check(&prev, CheckOutcome::Available { version: "0.1.1".into(), notes: "n".into() }, false, 99),
            UpdateStatus::Available { version: "0.1.1".into(), notes: "n".into() }
        );
    }

    #[test]
    fn a_failed_check_is_shown_only_when_the_user_started_it() {
        let prev = UpdateStatus::UpToDate { checked_at: 5 };
        assert_eq!(after_check(&prev, CheckOutcome::Failed("offline".into()), false, 99), prev);
        assert_eq!(
            after_check(&prev, CheckOutcome::Failed("offline".into()), true, 99),
            UpdateStatus::Error { message: "offline".into(), during: "check".into() }
        );
    }
}
