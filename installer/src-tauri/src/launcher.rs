//! Runs before any window exists, so it must not need WebView2: checks for the WebView2 runtime and,
//! when it is missing, downloads and installs Microsoft's Evergreen bootstrapper.
use crate::logic::{pick_lang, webview2_present, Lang};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK};
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Verified on a real PC (docs/superpowers/notes/2026-11-02-installer-facts.md).
const CLIENT_KEY: &str = r"Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
const BOOTSTRAPPER_URL: &str = "https://go.microsoft.com/fwlink/p/?LinkId=2124703";
const MANUAL_URL: &str = "https://developer.microsoft.com/microsoft-edge/webview2/";

/// The machine-wide value wins; an empty value counts as missing.
pub fn first_pv(machine: Option<String>, user: Option<String>) -> Option<String> {
    machine.filter(|v| !v.is_empty()).or(user)
}

fn read_pv(root: &RegKey, subkey: &str) -> Option<String> {
    root.open_subkey(subkey).ok()?.get_value::<String, _>("pv").ok()
}

pub fn webview2_installed() -> bool {
    let machine = read_pv(&RegKey::predef(HKEY_LOCAL_MACHINE), &format!(r"SOFTWARE\WOW6432Node\{CLIENT_KEY}"));
    let user = read_pv(&RegKey::predef(HKEY_CURRENT_USER), &format!(r"Software\{CLIENT_KEY}"));
    webview2_present(first_pv(machine, user).as_deref())
}

pub fn download_args(url: &str, target: &Path) -> Vec<String> {
    vec!["-sS".into(), "-L".into(), "--fail".into(), "-o".into(), target.display().to_string(), url.into()]
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn show_message(title: &str, text: &str, error: bool) {
    let (t, m) = (wide(title), wide(text));
    let icon = if error { MB_ICONERROR } else { MB_ICONINFORMATION };
    unsafe {
        MessageBoxW(None, PCWSTR(m.as_ptr()), PCWSTR(t.as_ptr()), MB_OK | icon);
    }
}

/// Windows tools are started by absolute path. A bare name would be searched in the installer's own folder
/// first, so a `curl.exe` planted next to it (for example in Downloads) would run instead of the real one.
pub fn system_tool(name: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    root.join("System32").join(name)
}

fn download(target: &Path) -> Result<(), String> {
    // curl.exe ships with Windows 10 and 11 (System32), follows the fwlink redirect and needs no extra crate.
    let status = Command::new(system_tool("curl.exe"))
        .args(download_args(BOOTSTRAPPER_URL, target))
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("download failed with code {}", status.code().unwrap_or(-1)))
    }
}

/// Installs WebView2 when it is missing. `MACDOWS_FORCE_NO_WEBVIEW2=1` only downloads the bootstrapper to
/// the temp folder and deletes it (used to exercise the download path on a PC that has WebView2).
pub fn ensure_webview2(lang: Lang) -> Result<(), String> {
    let force = std::env::var_os("MACDOWS_FORCE_NO_WEBVIEW2").is_some();
    if webview2_installed() && !force {
        return Ok(());
    }
    let (title, intro) = match lang {
        Lang::Tr => ("MacDows Kurulumu", "MacDows kurulumu için gerekli bir bileşen (WebView2) indirilip kurulacak. Bu işlem internet bağlantısı gerektirir ve birkaç dakika sürebilir. Başlamak için Tamam'a basın."),
        Lang::En => ("MacDows Setup", "A required component (WebView2) will be downloaded and installed first. This needs an internet connection and can take a few minutes. Press OK to start."),
    };
    if !force {
        show_message(title, intro, false);
    }
    let target: PathBuf = std::env::temp_dir().join("MicrosoftEdgeWebview2Setup.exe");
    download(&target).map_err(|e| format!("{e} ({MANUAL_URL})"))?;
    if force {
        let _ = std::fs::remove_file(&target);
        return Ok(());
    }
    let status = Command::new(&target)
        .args(["/silent", "/install"])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&target);
    if !status.success() || !webview2_installed() {
        return Err(format!("WebView2 setup stopped with code {} ({MANUAL_URL})", status.code().unwrap_or(-1)));
    }
    Ok(())
}

/// Entry point called from `main` before the window is built. Returns false when the installer must stop.
pub fn prepare(locale: &str) -> bool {
    let lang = pick_lang(locale);
    match ensure_webview2(lang) {
        Ok(()) => true,
        Err(reason) => {
            let (title, text) = match lang {
                Lang::Tr => ("MacDows Kurulumu", format!("Gerekli bileşen (WebView2) kurulamadı.\n\n{reason}")),
                Lang::En => ("MacDows Setup", format!("The required component (WebView2) could not be installed.\n\n{reason}")),
            };
            show_message(title, &text, true);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_value_wins_over_user_value() {
        assert_eq!(first_pv(Some("1.0".into()), Some("2.0".into())), Some("1.0".into()));
        assert_eq!(first_pv(None, Some("2.0".into())), Some("2.0".into()));
        assert_eq!(first_pv(None, None), None);
    }

    #[test]
    fn an_empty_machine_value_falls_through_to_the_user_value() {
        assert_eq!(first_pv(Some("".into()), Some("2.0".into())), Some("2.0".into()));
    }

    /// Needs internet. Run with `cargo test -- --ignored`. Exercises the real download without installing anything.
    #[test]
    #[ignore]
    fn forced_download_path_downloads_and_cleans_up() {
        std::env::set_var("MACDOWS_FORCE_NO_WEBVIEW2", "1");
        assert!(ensure_webview2(Lang::En).is_ok());
        assert!(!std::env::temp_dir().join("MicrosoftEdgeWebview2Setup.exe").exists());
    }

    #[test]
    fn system_tools_are_started_by_absolute_system32_path() {
        let curl = system_tool("curl.exe");
        assert!(curl.is_absolute(), "{curl:?} must be absolute so a planted curl.exe next to the installer is never used");
        assert!(curl.to_string_lossy().to_lowercase().ends_with(r"system32\curl.exe"));
    }

    #[test]
    fn download_command_follows_redirects_and_names_the_target() {
        let args = download_args("https://example.test/x.exe", std::path::Path::new(r"C:\Temp\Çağrı\x.exe"));
        assert!(args.contains(&"-L".to_string()));
        assert!(args.contains(&"--fail".to_string()));
        assert_eq!(args.last().unwrap(), "https://example.test/x.exe");
        assert!(args.iter().any(|a| a == r"C:\Temp\Çağrı\x.exe"));
    }
}
