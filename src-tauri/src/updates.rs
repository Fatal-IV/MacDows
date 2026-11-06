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

/// The heading that starts the Turkish part of release notes (the English part comes first).
const TURKISH_MARKER: &str = "### Türkçe";

/// The part of two-language release notes that matches `language`: Turkish after the "### Türkçe" heading for
/// "tr", the English text before it otherwise. Notes without the heading are returned whole.
pub fn notes_for_language(notes: &str, language: &str) -> String {
    let Some(at) = notes.lines().position(|l| l.trim() == TURKISH_MARKER) else {
        return notes.to_string();
    };
    let lines: Vec<&str> = notes.lines().collect();
    let part = if language == "tr" { &lines[at + 1..] } else { &lines[..at] };
    part.join("\n").trim().to_string()
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

// ---------------------------------------------------------------------------------------------
// Effects. Error messages sent to the UI are stable codes (`check_failed`, `download_failed`,
// `verify_failed`, `install_failed`, `no_update`, `busy`); the UI shows the localized sentence and
// the technical detail goes to the log.
// ---------------------------------------------------------------------------------------------
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::UpdaterExt;

/// How often the background thread wakes up to ask `should_check`.
const POLL_SECS: u64 = 1800;

pub struct UpdateState(pub Mutex<UpdateStatus>);

impl Default for UpdateState {
    fn default() -> Self {
        UpdateState(Mutex::new(UpdateStatus::Idle))
    }
}

/// Moves to `Checking` unless a check or install is already running.
pub fn begin_check(state: &Mutex<UpdateStatus>) -> bool {
    let mut status = state.lock().unwrap();
    if !can_start(&status) {
        return false;
    }
    *status = UpdateStatus::Checking;
    true
}

/// Moves to `Installing` only from `Available`; a second install can never start.
pub fn begin_install(state: &Mutex<UpdateStatus>) -> bool {
    let mut status = state.lock().unwrap();
    if !matches!(*status, UpdateStatus::Available { .. }) {
        return false;
    }
    *status = UpdateStatus::Installing { percent: None };
    true
}

pub fn classify_install_error(error: &tauri_plugin_updater::Error) -> &'static str {
    use tauri_plugin_updater::Error::*;
    match error {
        Minisign(_) | Base64(_) | SignatureUtf8(_) | SignedVersionMismatch { .. } | MissingSignedVersion => "verify_failed",
        Reqwest(_) | Network(_) => "download_failed",
        _ => "install_failed",
    }
}

fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn log(message: &str) {
    use std::io::Write;
    let base = std::env::var_os("APPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let dir = base.join("MacDows").join("logs");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("updates.log")) {
        let _ = writeln!(file, "[{}] {message}", now_secs());
    }
}

fn publish(app: &AppHandle, status: UpdateStatus) -> UpdateStatus {
    *app.state::<UpdateState>().0.lock().unwrap() = status.clone();
    let _ = app.emit("update-status", status.clone());
    status
}

fn build_updater(app: &AppHandle) -> Result<tauri_plugin_updater::Updater, String> {
    #[allow(unused_mut)]
    let mut builder = app.updater_builder();
    // Debug builds only: point the check at a local fake release (the plugin refuses http in release builds anyway).
    #[cfg(debug_assertions)]
    if let Ok(endpoint) = std::env::var("MACDOWS_UPDATE_ENDPOINT") {
        let url: tauri::Url = endpoint.parse().map_err(|e| format!("bad MACDOWS_UPDATE_ENDPOINT: {e}"))?;
        builder = builder.endpoints(vec![url]).map_err(|e| e.to_string())?;
    }
    builder.build().map_err(|e| e.to_string())
}

fn remember_check(app: &AppHandle, now: u64) {
    let settings = app.state::<crate::AppSettings>();
    let mut guard = settings.0.lock().unwrap();
    guard.last_update_check = Some(now);
    let _ = crate::settings::save(&guard);
}

/// Runs one check. `user_initiated` decides whether a failure becomes visible.
pub async fn run_check(app: &AppHandle, user_initiated: bool) -> UpdateStatus {
    let state = app.state::<UpdateState>();
    // The status before the check is what a silent failure falls back to.
    let previous = state.0.lock().unwrap().clone();
    if !begin_check(&state.0) {
        return previous;
    }
    let _ = app.emit("update-status", UpdateStatus::Checking);
    let outcome = match build_updater(app) {
        Err(e) => {
            log(&format!("check: could not build the updater: {e}"));
            CheckOutcome::Failed("check_failed".into())
        }
        Ok(updater) => match updater.check().await {
            Ok(Some(update)) => {
                let language = app.state::<crate::AppSettings>().0.lock().unwrap().language.clone();
                CheckOutcome::Available { version: update.version.clone(), notes: notes_for_language(&update.body.clone().unwrap_or_default(), &language) }
            }
            Ok(None) => CheckOutcome::UpToDate,
            Err(e) => {
                log(&format!("check failed: {e}"));
                CheckOutcome::Failed("check_failed".into())
            }
        },
    };
    let now = now_secs();
    if !matches!(outcome, CheckOutcome::Failed(_)) {
        remember_check(app, now);
    }
    publish(app, after_check(&previous, outcome, user_initiated, now))
}

async fn download_and_install(app: &AppHandle) -> Result<(), &'static str> {
    let updater = build_updater(app).map_err(|e| {
        log(&format!("install: could not build the updater: {e}"));
        "check_failed"
    })?;
    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => return Err("no_update"),
        Err(e) => {
            log(&format!("install: re-check failed: {e}"));
            return Err("check_failed");
        }
    };
    let mut downloaded: u64 = 0;
    // The release carries MacDows's own installer; the plugin checks its signature, then the installer's window
    // (not a plain NSIS one) shows the update.
    let bytes = update
        .download(
            |chunk, total| {
                downloaded += chunk as u64;
                let percent = total.filter(|t| *t > 0).map(|t| ((downloaded * 100) / t).min(100) as u8);
                publish(app, UpdateStatus::Installing { percent });
            },
            || log("download finished, starting the installer"),
        )
        .await
        .map_err(|e| {
            log(&format!("install failed: {e}"));
            classify_install_error(&e)
        })?;
    let dir = std::env::temp_dir().join("MacDows-Update");
    let file = dir.join("MacDows-Setup.exe");
    std::fs::create_dir_all(&dir)
        .and_then(|_| std::fs::write(&file, &bytes))
        .and_then(|_| std::process::Command::new(&file).arg("--update").spawn().map(|_| ()))
        .map_err(|e| {
            log(&format!("could not start the installer: {e}"));
            "install_failed"
        })?;
    // The installer replaces the files of this running program: step aside.
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn get_update_status(state: tauri::State<UpdateState>) -> UpdateStatus {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> UpdateStatus {
    run_check(&app, true).await
}

/// Installs the update that is `Available`. On success MacDows's installer window takes over and this process exits.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    if !begin_install(&app.state::<UpdateState>().0) {
        return Err("busy".into());
    }
    publish(&app, UpdateStatus::Installing { percent: None });
    match download_and_install(&app).await {
        Ok(()) => Ok(()),
        Err(code) => {
            publish(&app, UpdateStatus::Error { message: code.to_string(), during: "install".into() });
            Err(code.to_string())
        }
    }
}

/// Background checks: first one `FIRST_CHECK_DELAY_SECS` after launch, then whenever `should_check` says so.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(FIRST_CHECK_DELAY_SECS));
        loop {
            let (enabled, last) = {
                let settings = app.state::<crate::AppSettings>();
                let guard = settings.0.lock().unwrap();
                (guard.check_updates, guard.last_update_check)
            };
            if enabled && should_check(last, now_secs(), CHECK_INTERVAL_SECS) {
                tauri::async_runtime::block_on(run_check(&app, false));
            }
            std::thread::sleep(Duration::from_secs(POLL_SECS));
        }
    });
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
    fn two_language_notes_show_the_part_for_the_language() {
        let notes = "### Added\n- Eyes\n\n### Türkçe\n### Eklendi\n- Gözler\n";
        assert_eq!(notes_for_language(notes, "tr"), "### Eklendi\n- Gözler");
        assert_eq!(notes_for_language(notes, "en"), "### Added\n- Eyes");
        assert_eq!(notes_for_language(notes, "de"), "### Added\n- Eyes");
    }

    #[test]
    fn notes_without_a_turkish_part_are_shown_whole_in_every_language() {
        assert_eq!(notes_for_language("### Added\n- Eyes", "tr"), "### Added\n- Eyes");
        assert_eq!(notes_for_language("", "tr"), "");
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

#[cfg(test)]
mod effect_tests {
    use super::*;
    use std::sync::Mutex;
    use tauri_plugin_updater::Error;

    fn available() -> UpdateStatus {
        UpdateStatus::Available { version: "0.1.1".into(), notes: String::new() }
    }

    #[test]
    fn install_starts_only_from_available_and_never_twice() {
        let state = Mutex::new(available());
        assert!(begin_install(&state));
        assert_eq!(*state.lock().unwrap(), UpdateStatus::Installing { percent: None });
        assert!(!begin_install(&state), "a second install must not start");
        assert_eq!(*state.lock().unwrap(), UpdateStatus::Installing { percent: None });

        let idle = Mutex::new(UpdateStatus::Idle);
        assert!(!begin_install(&idle), "nothing to install when no update is known");
        assert_eq!(*idle.lock().unwrap(), UpdateStatus::Idle);
    }

    #[test]
    fn a_check_does_not_start_while_one_runs() {
        let state = Mutex::new(UpdateStatus::Idle);
        assert!(begin_check(&state));
        assert_eq!(*state.lock().unwrap(), UpdateStatus::Checking);
        assert!(!begin_check(&state));
        let installing = Mutex::new(UpdateStatus::Installing { percent: Some(5) });
        assert!(!begin_check(&installing));
        assert_eq!(*installing.lock().unwrap(), UpdateStatus::Installing { percent: Some(5) });
    }

    #[test]
    fn install_errors_are_classified_for_the_user() {
        assert_eq!(classify_install_error(&Error::SignatureUtf8("x".into())), "verify_failed");
        assert_eq!(classify_install_error(&Error::Network("x".into())), "download_failed");
        assert_eq!(classify_install_error(&Error::EmptyEndpoints), "install_failed");
    }
}
