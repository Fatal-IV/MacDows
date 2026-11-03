//! The quick settings of the notch: Wi-Fi, Bluetooth, Do not disturb and Battery saver.
//!
//! - Wi-Fi and Bluetooth go through the Windows radio API (`Windows.Devices.Radios`).
//! - Do not disturb switches the notification banners off and on (`ToastEnabled`); it does not drive
//!   Windows's own Focus assist, which has no supported API.
//! - Battery saver sets the battery-saver threshold to 100 % (always on while on battery) or 0 % (off).

use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::process::Command;
use windows::core::w;
use windows::Devices::Radios::{Radio, RadioAccessStatus, RadioKind, RadioState};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
use windows::Win32::System::Registry::{RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_DWORD, RRF_RT_REG_DWORD};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// One tile: `available` is false when the PC has no such hardware, `on` is its current state.
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct Tile {
    pub available: bool,
    pub on: bool,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Default)]
pub struct QuickSettings {
    pub wifi: Tile,
    pub bluetooth: Tile,
    pub dnd: Tile,
    pub battery_saver: Tile,
}

/// The last `0x…` number in `powercfg /query` output. The text around it is localized, the number is not;
/// the DC (battery) value is printed last.
pub fn last_hex_value(output: &str) -> Option<u32> {
    output
        .split_whitespace()
        .filter_map(|w| w.strip_prefix("0x"))
        .filter_map(|h| u32::from_str_radix(h, 16).ok())
        .last()
}

/// Battery saver counts as on when it starts at a threshold above 0 % while on battery.
pub fn saver_on(threshold: u32) -> bool {
    threshold > 0
}

/// `BatteryFlag` 128 is "no system battery" and 255 is "unknown".
pub fn has_battery(flag: u8) -> bool {
    flag != 128 && flag != 255
}

fn radio_tile(kind: RadioKind) -> Tile {
    let Ok(op) = Radio::GetRadiosAsync() else { return Tile::default() };
    let Ok(radios) = op.get() else { return Tile::default() };
    for radio in radios {
        if radio.Kind().ok() == Some(kind) {
            return Tile { available: true, on: radio.State().ok() == Some(RadioState::On) };
        }
    }
    Tile::default()
}

fn set_radio(kind: RadioKind, on: bool) -> Result<(), String> {
    let access = Radio::RequestAccessAsync().and_then(|op| op.get()).map_err(|e| e.message())?;
    if access != RadioAccessStatus::Allowed {
        return Err("Windows did not allow changing the radio".into());
    }
    let radios = Radio::GetRadiosAsync().and_then(|op| op.get()).map_err(|e| e.message())?;
    for radio in radios {
        if radio.Kind().ok() == Some(kind) {
            let state = if on { RadioState::On } else { RadioState::Off };
            radio.SetStateAsync(state).and_then(|op| op.get()).map_err(|e| e.message())?;
        }
    }
    Ok(())
}

const TOAST_KEY: windows::core::PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\PushNotifications");

fn banners_on() -> bool {
    let mut value: u32 = 1; // the value is absent while banners are on
    let mut size = std::mem::size_of::<u32>() as u32;
    let status = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, TOAST_KEY, w!("ToastEnabled"), RRF_RT_REG_DWORD, None, Some(&mut value as *mut u32 as *mut _), Some(&mut size))
    };
    status != ERROR_SUCCESS || value != 0
}

fn set_banners(on: bool) -> Result<(), String> {
    let value: u32 = on as u32;
    let status = unsafe {
        RegSetKeyValueW(HKEY_CURRENT_USER, TOAST_KEY, w!("ToastEnabled"), REG_DWORD.0, Some(&value as *const u32 as *const _), 4)
    };
    if status == ERROR_SUCCESS { Ok(()) } else { Err(format!("registry error {}", status.0)) }
}

fn powercfg(args: &[&str]) -> Option<String> {
    let out = Command::new("powercfg").args(args).creation_flags(CREATE_NO_WINDOW).output().ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn saver_tile() -> Tile {
    let mut status = SYSTEM_POWER_STATUS::default();
    if unsafe { GetSystemPowerStatus(&mut status) }.is_err() || !has_battery(status.BatteryFlag) {
        return Tile::default();
    }
    let on = powercfg(&["/query", "SCHEME_CURRENT", "SUB_ENERGYSAVER", "ESBATTTHRESHOLD"])
        .and_then(|o| last_hex_value(&o))
        .map(saver_on)
        .unwrap_or(false);
    Tile { available: true, on }
}

fn set_saver(on: bool) -> Result<(), String> {
    let level = if on { "100" } else { "0" };
    powercfg(&["/setdcvalueindex", "SCHEME_CURRENT", "SUB_ENERGYSAVER", "ESBATTTHRESHOLD", level]).ok_or("powercfg failed")?;
    powercfg(&["/setactive", "SCHEME_CURRENT"]).ok_or("powercfg failed")?;
    Ok(())
}

fn read_all() -> QuickSettings {
    QuickSettings {
        wifi: radio_tile(RadioKind::WiFi),
        bluetooth: radio_tile(RadioKind::Bluetooth),
        dnd: Tile { available: true, on: !banners_on() },
        battery_saver: saver_tile(),
    }
}

/// A slider value as a whole percent: NaN becomes 0, anything out of range is clamped.
pub fn clamp_percent(level: f64) -> u32 {
    if level.is_nan() { 0 } else { level.clamp(0.0, 100.0).round() as u32 }
}

fn powershell(script: &str) -> Option<String> {
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Brightness of the built-in display (0 to 100); `None` on a PC whose screen has no brightness control.
#[tauri::command]
pub async fn get_brightness() -> Option<u32> {
    tauri::async_runtime::spawn_blocking(|| {
        powershell("(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1).CurrentBrightness")?
            .trim()
            .parse()
            .ok()
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
pub async fn set_brightness(level: f64) -> Result<(), String> {
    let percent = clamp_percent(level);
    tauri::async_runtime::spawn_blocking(move || {
        powershell(&format!(
            "Invoke-CimMethod -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods -MethodName WmiSetBrightness -Arguments @{{Timeout=1;Brightness={percent}}} -ErrorAction Stop | Out-Null"
        ))
        .map(|_| ())
        .ok_or_else(|| "could not set the brightness".to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// These wait on WinRT and child processes, so they run off the main thread.
#[tauri::command]
pub async fn get_quick_settings() -> Result<QuickSettings, String> {
    tauri::async_runtime::spawn_blocking(read_all).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_quick_setting(key: String, on: bool) -> Result<QuickSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        match key.as_str() {
            "wifi" => set_radio(RadioKind::WiFi, on)?,
            "bluetooth" => set_radio(RadioKind::Bluetooth, on)?,
            "dnd" => set_banners(!on)?,
            "battery_saver" => set_saver(on)?,
            _ => return Err(format!("unknown quick setting: {key}")),
        }
        Ok(read_all())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn takes_the_last_hex_number_whatever_the_language() {
        let out = "Güç Ayarı Dizini: 0x00000000\n  Geçerli AC Güç Ayarı Dizini: 0x00000000\n  Geçerli DC Güç Ayarı Dizini: 0x00000064\n";
        assert_eq!(last_hex_value(out), Some(100));
        assert_eq!(last_hex_value("no numbers here"), None);
    }

    #[test]
    fn percent_is_clamped() {
        assert_eq!(clamp_percent(f64::NAN), 0);
        assert_eq!(clamp_percent(-5.0), 0);
        assert_eq!(clamp_percent(42.4), 42);
        assert_eq!(clamp_percent(180.0), 100);
    }

    #[test]
    fn saver_is_on_above_zero() {
        assert!(saver_on(100));
        assert!(saver_on(20));
        assert!(!saver_on(0));
    }

    #[test]
    fn battery_flags() {
        assert!(!has_battery(128));
        assert!(!has_battery(255));
        assert!(has_battery(1));
        assert!(has_battery(8));
    }
}
