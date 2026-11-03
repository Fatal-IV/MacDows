//! System output volume, through the default playback device's endpoint volume.

use serde::Serialize;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{eMultimedia, eRender, IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

#[derive(Serialize, Clone, Copy, Debug, PartialEq)]
pub struct Volume {
    /// 0.0 to 1.0
    pub level: f32,
    pub muted: bool,
}

/// A slider value as a safe level: NaN becomes 0 and anything out of range is clamped.
pub fn clamp_level(level: f32) -> f32 {
    if level.is_nan() { 0.0 } else { level.clamp(0.0, 1.0) }
}

fn endpoint() -> Option<IAudioEndpointVolume> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?;
        device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).ok()
    }
}

fn read() -> Option<Volume> {
    let ep = endpoint()?;
    unsafe { Some(Volume { level: ep.GetMasterVolumeLevelScalar().ok()?, muted: ep.GetMute().ok()?.as_bool() }) }
}

fn write(level: f32) -> Option<Volume> {
    let ep = endpoint()?;
    unsafe {
        ep.SetMasterVolumeLevelScalar(clamp_level(level), std::ptr::null()).ok()?;
        // Moving the slider is a clear wish to hear something: undo a mute.
        if clamp_level(level) > 0.0 && ep.GetMute().ok()?.as_bool() {
            ep.SetMute(false, std::ptr::null()).ok()?;
        }
    }
    read()
}

// Async on purpose: these talk to COM and must not run on the main thread.
#[tauri::command]
pub async fn get_volume() -> Option<Volume> {
    tauri::async_runtime::spawn_blocking(read).await.ok().flatten()
}

#[tauri::command]
pub async fn set_volume(level: f32) -> Option<Volume> {
    tauri::async_runtime::spawn_blocking(move || write(level)).await.ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_are_clamped_to_zero_and_one() {
        assert_eq!(clamp_level(0.5), 0.5);
        assert_eq!(clamp_level(-3.0), 0.0);
        assert_eq!(clamp_level(7.0), 1.0);
        assert_eq!(clamp_level(f32::NAN), 0.0);
    }
}
