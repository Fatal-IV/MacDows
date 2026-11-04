//! Drives for the dock: local disks and removable drives, opened in File Explorer. Removable ones come and go.
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Drive {
    /// "C" (no colon).
    pub letter: String,
    /// "local" or "removable".
    pub kind: String,
    /// The volume's name; "" when it has none (the dock then shows a generic name).
    pub label: String,
}

/// Only local disks and removable drives belong in the dock (not network, CD or RAM drives).
pub fn kind_of(drive_type: u32) -> Option<&'static str> {
    match drive_type {
        DRIVE_FIXED => Some("local"),
        DRIVE_REMOVABLE => Some("removable"),
        _ => None,
    }
}

/// The letters whose bit is set in the mask `GetLogicalDrives` returns (bit 0 is A).
pub fn letters(mask: u32) -> Vec<char> {
    (0..26u32).filter(|bit| mask & (1 << bit) != 0).map(|bit| (b'A' + bit as u8) as char).collect()
}

/// A single letter A to Z (any case), nothing else.
pub fn valid_letter(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let c = chars.next()?;
    (chars.next().is_none() && c.is_ascii_alphabetic()).then(|| c.to_ascii_uppercase())
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

/// The drives that can be opened right now (a card reader without a card is left out).
fn read_drives() -> Vec<Drive> {
    let mask = unsafe { GetLogicalDrives() };
    letters(mask)
        .into_iter()
        .filter_map(|letter| {
            let root = wide(&format!("{letter}:\\"));
            let kind = kind_of(unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) })?;
            let mut name = [0u16; 261];
            let ready = unsafe { GetVolumeInformationW(PCWSTR(root.as_ptr()), Some(&mut name), None, None, None, None) }.is_ok();
            if !ready {
                return None;
            }
            let len = name.iter().position(|&c| c == 0).unwrap_or(0);
            Some(Drive { letter: letter.to_string(), kind: kind.to_string(), label: String::from_utf16_lossy(&name[..len]) })
        })
        .collect()
}

fn enabled(app: &AppHandle) -> bool {
    app.state::<crate::AppSettings>().0.lock().unwrap().dock.show_drives
}

/// Empty when drives are switched off.
#[tauri::command]
pub async fn list_drives(app: AppHandle) -> Vec<Drive> {
    if !enabled(&app) {
        return Vec::new();
    }
    tauri::async_runtime::spawn_blocking(read_drives).await.unwrap_or_default()
}

#[tauri::command]
pub fn open_drive(letter: String) -> Result<(), String> {
    let letter = valid_letter(&letter).ok_or("not a drive letter")?;
    let root = wide(&format!("{letter}:\\"));
    unsafe {
        ShellExecuteW(None, windows::core::w!("open"), PCWSTR(root.as_ptr()), PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL);
    }
    Ok(())
}

/// Tells the dock when a drive appears or goes (checked every 3 seconds, and only while drives are shown).
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last = Vec::new();
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            if !enabled(&app) {
                last.clear();
                continue;
            }
            let now = read_drives();
            if now != last {
                last = now;
                let _ = app.emit("drives-changed", ());
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_local_and_removable_drives_count() {
        assert_eq!(kind_of(3), Some("local"));
        assert_eq!(kind_of(2), Some("removable"));
        for other in [0, 1, 4, 5, 6] {
            assert_eq!(kind_of(other), None, "drive type {other}");
        }
    }

    #[test]
    fn letters_come_from_the_bits() {
        assert_eq!(letters(0b101), vec!['A', 'C']);
        assert_eq!(letters(0), Vec::<char>::new());
        assert_eq!(letters(1 << 25), vec!['Z']);
    }

    #[test]
    fn only_a_single_letter_opens_a_drive() {
        assert_eq!(valid_letter("c"), Some('C'));
        assert_eq!(valid_letter("D"), Some('D'));
        for bad in ["", "1", "CC", "C:", " ", "é"] {
            assert_eq!(valid_letter(bad), None, "{bad:?}");
        }
        assert!(open_drive("1".into()).is_err());
    }

    #[test]
    fn this_pc_has_a_local_system_drive() {
        // Real call: any Windows machine this runs on has a ready fixed C: drive.
        assert!(read_drives().iter().any(|d| d.kind == "local"));
    }
}
