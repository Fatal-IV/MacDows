//! Windows effects of the installer: extract the embedded MacDows NSIS package, run it silently,
//! read what is installed, start the app. Decisions live in `logic.rs`.
use crate::logic::{decide, exit_code_message, log_line, pick_lang, stage_percent, Lang, Stage};
use serde::Serialize;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use windows::Win32::Globalization::GetUserDefaultLocaleName;
use windows::Win32::System::SystemInformation::GetLocalTime;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Verified on a real install (docs/superpowers/notes/2026-11-02-installer-facts.md).
const UNINSTALL_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\MacDows";
static PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.exe"));
/// The MacDows version that the embedded package installs (injected by scripts/build-installer.mjs).
pub const BUNDLED_VERSION: &str = match option_env!("MACDOWS_APP_VERSION") {
    Some(v) => v,
    None => env!("CARGO_PKG_VERSION"),
};

#[derive(Debug)]
pub enum SetupError {
    PayloadMissing,
    Io(String),
    NotInstalled,
}

impl SetupError {
    pub fn user_message(&self, lang: Lang) -> String {
        match (self, lang) {
            (SetupError::PayloadMissing, Lang::Tr) => "Kurulum dosyası pakette bulunamadı.".into(),
            (SetupError::PayloadMissing, Lang::En) => "The setup package is missing from this installer.".into(),
            (SetupError::Io(e), Lang::Tr) => format!("Dosya işlemi başarısız oldu: {e}"),
            (SetupError::Io(e), Lang::En) => format!("A file operation failed: {e}"),
            (SetupError::NotInstalled, Lang::Tr) => "MacDows kurulu görünmüyor.".into(),
            (SetupError::NotInstalled, Lang::En) => "MacDows does not appear to be installed.".into(),
        }
    }
}

fn io(e: std::io::Error) -> SetupError {
    SetupError::Io(e.to_string())
}

/// The build embeds a tiny stub when no real package was provided; real packages are megabytes.
fn is_stub_len(len: usize) -> bool {
    len < 1024
}

pub fn payload_is_stub() -> bool {
    is_stub_len(PAYLOAD.len())
}

pub fn extract_payload(dest_dir: &Path) -> Result<PathBuf, SetupError> {
    std::fs::create_dir_all(dest_dir).map_err(io)?;
    let exe = dest_dir.join("MacDows-setup.exe");
    std::fs::write(&exe, PAYLOAD).map_err(io)?;
    Ok(exe)
}

/// Runs the NSIS package without any window and returns its exit code.
pub fn run_nsis(exe: &Path) -> Result<i32, SetupError> {
    let status = Command::new(exe).arg("/S").creation_flags(CREATE_NO_WINDOW).status().map_err(io)?;
    Ok(status.code().unwrap_or(-1))
}

pub struct InstalledInfo {
    pub version: String,
    pub location: PathBuf,
}

/// The NSIS package writes `InstallLocation` with literal double quotes.
pub fn parse_install_location(raw: &str) -> PathBuf {
    PathBuf::from(raw.trim().trim_matches('"'))
}

pub fn read_installed() -> Option<InstalledInfo> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(UNINSTALL_KEY).ok()?;
    let version: String = key.get_value("DisplayVersion").ok()?;
    let location: String = key.get_value("InstallLocation").ok()?;
    Some(InstalledInfo { version, location: parse_install_location(&location) })
}

pub fn system_locale() -> String {
    let mut buf = [0u16; 85];
    let n = unsafe { GetUserDefaultLocaleName(&mut buf) };
    if n > 1 {
        String::from_utf16_lossy(&buf[..(n as usize - 1)])
    } else {
        String::new()
    }
}

fn timestamp() -> String {
    let t = unsafe { GetLocalTime() };
    format!("{:02}:{:02}:{:02}", t.wHour, t.wMinute, t.wSecond)
}

pub fn write_log_to(file: &Path, level: &str, message: &str) {
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(file) {
        let _ = writeln!(f, "{}", log_line(&timestamp(), level, message));
    }
}

pub fn log_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("MacDows-Setup").join("setup.log")
}

fn write_log(level: &str, message: &str) {
    write_log_to(&log_path(), level, message);
}

#[derive(Serialize)]
pub struct InstallInfo {
    pub decision: String,
    pub installed_version: Option<String>,
    pub bundled_version: String,
    pub lang: String,
    pub log_path: String,
    /// MacDows started this installer to update itself (`--update`): no questions, install and reopen the app.
    pub auto_update: bool,
}

#[tauri::command]
pub fn install_state() -> InstallInfo {
    let installed = read_installed();
    let decision = decide(installed.as_ref().map(|i| i.version.as_str()), BUNDLED_VERSION);
    let lang = if pick_lang(&system_locale()) == Lang::Tr { "tr" } else { "en" };
    InstallInfo {
        decision: decision.as_str().to_string(),
        installed_version: installed.map(|i| i.version),
        bundled_version: BUNDLED_VERSION.to_string(),
        lang: lang.to_string(),
        log_path: log_path().display().to_string(),
        auto_update: std::env::args().any(|a| a == "--update"),
    }
}

#[derive(Serialize, Clone)]
struct StageEvent {
    stage: &'static str,
    percent: u8,
}

#[derive(Serialize, Clone)]
struct ErrorEvent {
    message: String,
}

fn emit_stage(app: &AppHandle, stage: Stage) {
    let (name, percent) = match stage {
        Stage::Preparing => ("preparing", stage_percent(stage)),
        Stage::Installing => ("installing", stage_percent(stage)),
        Stage::Finishing => ("finishing", stage_percent(stage)),
        Stage::Done => ("done", stage_percent(stage)),
    };
    write_log("info", &format!("stage {name} {percent}%"));
    let _ = app.emit("install-stage", StageEvent { stage: name, percent });
}

fn fail(app: &AppHandle, message: String) {
    write_log("error", &message);
    let _ = app.emit("install-error", ErrorEvent { message });
}

#[tauri::command]
pub fn start_install(app: AppHandle, autostart: Option<bool>) -> Result<(), String> {
    let lang = pick_lang(&system_locale());
    if payload_is_stub() {
        let msg = SetupError::PayloadMissing.user_message(lang);
        write_log("error", &msg);
        return Err(msg);
    }
    std::thread::spawn(move || run_install(&app, lang, autostart));
    Ok(())
}

fn run_install(app: &AppHandle, lang: Lang, autostart: Option<bool>) {
    write_log("info", &format!("install started, bundled version {BUNDLED_VERSION}"));
    emit_stage(app, Stage::Preparing);
    let dir = std::env::temp_dir().join("MacDows-Setup");
    let exe = match extract_payload(&dir) {
        Ok(p) => p,
        Err(e) => return fail(app, e.user_message(lang)),
    };
    emit_stage(app, Stage::Installing);
    // Silent NSIS reports no progress: creep slowly from the Installing stage towards 90% while it runs.
    let stop = Arc::new(AtomicBool::new(false));
    let creeper = {
        let (app, stop) = (app.clone(), stop.clone());
        std::thread::spawn(move || {
            let mut percent = stage_percent(Stage::Installing);
            while !stop.load(Ordering::Relaxed) && percent < 90 {
                std::thread::sleep(Duration::from_millis(500));
                percent += 1;
                let _ = app.emit("install-stage", StageEvent { stage: "installing", percent });
            }
        })
    };
    let result = run_nsis(&exe);
    stop.store(true, Ordering::Relaxed);
    let _ = creeper.join();
    let _ = std::fs::remove_dir_all(&dir);
    match result {
        Ok(0) => {
            if let Some(on) = autostart {
                apply_autostart_choice(on);
            }
            emit_stage(app, Stage::Finishing);
            emit_stage(app, Stage::Done);
            let _ = app.emit("install-done", ());
        }
        Ok(code) => fail(app, exit_code_message(lang, code)),
        Err(e) => fail(app, e.user_message(lang)),
    }
}

/// MacDows's settings file with `start_with_windows` set, keeping everything else that was in it.
pub fn settings_with_autostart(existing: Option<&str>, on: bool) -> String {
    let mut value = existing.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok()).filter(|v| v.is_object()).unwrap_or_else(|| serde_json::json!({}));
    value["start_with_windows"] = serde_json::Value::Bool(on);
    serde_json::to_string_pretty(&value).unwrap_or_default()
}

/// The Run entry value: the program path in double quotes.
pub fn run_value(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

/// Saves the "Start with Windows" choice for the new install: in MacDows's settings (which it reads at startup) and
/// in the Run key at once, so it does not depend on the first launch.
fn apply_autostart_choice(on: bool) {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let file = base.join("MacDows").join("settings.json");
    let existing = std::fs::read_to_string(&file).ok();
    if let Some(dir) = file.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    match std::fs::write(&file, settings_with_autostart(existing.as_deref(), on)) {
        Ok(()) => write_log("info", &format!("start with Windows: {on}")),
        Err(e) => write_log("error", &format!("could not save the start with Windows choice: {e}")),
    }
    let Some(info) = read_installed() else { return };
    let run = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Microsoft\Windows\CurrentVersion\Run", winreg::enums::KEY_SET_VALUE);
    if let Ok(run) = run {
        if on {
            let _ = run.set_value("MacDows", &run_value(&info.location.join("macdows.exe")));
        } else {
            let _ = run.delete_value("MacDows");
        }
    }
}

#[tauri::command]
pub fn launch_app() -> Result<(), String> {
    let lang = pick_lang(&system_locale());
    let info = read_installed().ok_or_else(|| SetupError::NotInstalled.user_message(lang))?;
    let exe = info.location.join("macdows.exe");
    write_log("info", &format!("launching {}", exe.display()));
    Command::new(&exe).spawn().map(|_| ()).map_err(|e| {
        let msg = SetupError::Io(e.to_string()).user_message(lang);
        write_log("error", &msg);
        msg
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn extract_payload_works_with_non_ascii_dir() {
        let dir = temp_dir("Çağrı Şen ı macdows-test");
        let exe = extract_payload(&dir).unwrap();
        assert!(exe.exists());
        assert_eq!(std::fs::metadata(&exe).unwrap().len(), PAYLOAD.len() as u64);
        assert!(exe.to_string_lossy().ends_with(".exe"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn log_file_is_appended() {
        let dir = temp_dir("macdows-log-test ş");
        let file = dir.join("setup.log");
        write_log_to(&file, "info", "first");
        write_log_to(&file, "error", "second");
        let text = std::fs::read_to_string(&file).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!(text.lines().next().unwrap().ends_with("INFO first"));
        assert!(text.lines().nth(1).unwrap().ends_with("ERROR second"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_location_quotes_are_stripped() {
        assert_eq!(
            parse_install_location(r#""C:\Users\fatal\AppData\Local\MacDows""#),
            PathBuf::from(r"C:\Users\fatal\AppData\Local\MacDows")
        );
        assert_eq!(parse_install_location(r"C:\Plain"), PathBuf::from(r"C:\Plain"));
    }

    #[test]
    fn the_autostart_choice_is_added_without_losing_other_settings() {
        let fresh: serde_json::Value = serde_json::from_str(&settings_with_autostart(None, true)).unwrap();
        assert_eq!(fresh, serde_json::json!({ "start_with_windows": true }));
        let kept: serde_json::Value = serde_json::from_str(&settings_with_autostart(Some(r#"{"language":"tr","start_with_windows":true}"#), false)).unwrap();
        assert_eq!(kept, serde_json::json!({ "language": "tr", "start_with_windows": false }));
        // A broken or non-object file is replaced rather than extended.
        let broken: serde_json::Value = serde_json::from_str(&settings_with_autostart(Some("not json"), true)).unwrap();
        assert_eq!(broken, serde_json::json!({ "start_with_windows": true }));
        let list: serde_json::Value = serde_json::from_str(&settings_with_autostart(Some("[1,2]"), true)).unwrap();
        assert_eq!(list, serde_json::json!({ "start_with_windows": true }));
    }

    #[test]
    fn the_run_entry_is_the_quoted_program_path() {
        assert_eq!(run_value(Path::new(r"C:\Users\fatal\AppData\Local\MacDows\macdows.exe")), r#""C:\Users\fatal\AppData\Local\MacDows\macdows.exe""#);
    }

    #[test]
    fn payloads_under_one_kilobyte_count_as_stubs() {
        assert!(is_stub_len(0));
        assert!(is_stub_len(1023));
        assert!(!is_stub_len(1024));
        assert!(!is_stub_len(2_400_000));
    }
}
