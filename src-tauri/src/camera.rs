//! Photo taken: the island shows a camera icon when the Windows Camera app saves a new picture.
//!
//! The Camera app writes every photo into the "Camera Roll" folder (Pictures\Camera Roll, or wherever the user moved
//! it). Nothing is read from the pictures: only the folder's time stamps are watched, and a new image file there
//! sends a `camera-shot` event to the notch.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter};
use windows::core::GUID;
use windows::Win32::UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT};

/// FOLDERID_CameraRoll.
const CAMERA_ROLL: GUID = GUID::from_u128(0xAB5FB87B_7CE2_4F83_915D_550846C9537B);

fn camera_roll() -> Option<PathBuf> {
    let known = unsafe { SHGetKnownFolderPath(&CAMERA_ROLL, KF_FLAG_DEFAULT, None) }.ok().and_then(|p| {
        let s = unsafe { p.to_string() }.ok();
        unsafe { windows::Win32::System::Com::CoTaskMemFree(Some(p.0 as _)) };
        s
    });
    let path = known.map(PathBuf::from).or_else(|| std::env::var_os("USERPROFILE").map(|h| Path::new(&h).join("Pictures").join("Camera Roll")))?;
    path.is_dir().then_some(path)
}

/// Whether `name` is a finished picture (the Camera app first writes a temporary file, then renames it).
pub fn is_photo(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    !lower.starts_with('.') && [".jpg", ".jpeg", ".png"].iter().any(|e| lower.ends_with(e))
}

/// The newest picture's time stamp in `dir`.
fn newest_photo(dir: &Path) -> Option<SystemTime> {
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| is_photo(&e.file_name().to_string_lossy()))
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .max()
}

/// Starts the watcher: the folder is only listed again when its own time stamp changes.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut seen: Option<SystemTime> = None; // newest picture known so far
        let mut dir_stamp: Option<SystemTime> = None;
        let mut baseline_taken = false;
        loop {
            std::thread::sleep(Duration::from_millis(700));
            let Some(dir) = camera_roll() else { continue };
            let stamp = std::fs::metadata(&dir).and_then(|m| m.modified()).ok();
            if baseline_taken && stamp == dir_stamp {
                continue;
            }
            dir_stamp = stamp;
            let newest = newest_photo(&dir);
            if baseline_taken && newest > seen {
                let _ = app.emit("camera-shot", ());
            }
            if newest > seen || !baseline_taken {
                seen = newest;
            }
            baseline_taken = true;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_finished_pictures_count() {
        assert!(is_photo("WIN_20261106_14_00_00_Pro.jpg"));
        assert!(is_photo("shot.PNG"));
        assert!(is_photo("a.jpeg"));
        assert!(!is_photo("clip.mp4"));
        assert!(!is_photo(".tmp123.jpg"));
        assert!(!is_photo("shot.jpg.tmp"));
    }

    #[test]
    fn this_pc_has_a_camera_roll_or_none() {
        // Must not panic whether or not the folder exists.
        let _ = camera_roll();
    }
}
