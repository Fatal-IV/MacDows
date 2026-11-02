use base64::Engine;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus, MediaPropertiesChangedEventArgs,
    PlaybackInfoChangedEventArgs,
};
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

#[derive(Debug, Clone, Serialize)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    /// Base64 of the artwork bytes (PNG or JPEG, see `artwork_mime`).
    pub artwork_png_base64: Option<String>,
    pub artwork_mime: String,
}

pub fn sniff_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else {
        "image/png"
    }
}

static APP: OnceLock<AppHandle> = OnceLock::new();
static MANAGER: OnceLock<Manager> = OnceLock::new();
static LAST: Mutex<Option<MediaInfo>> = Mutex::new(None);
static ATTACHED: Mutex<Option<(Session, i64, i64)>> = Mutex::new(None);

fn read_thumbnail(r: IRandomAccessStreamReference) -> Option<Vec<u8>> {
    let stream = r.OpenReadAsync().ok()?.get().ok()?;
    let size = stream.Size().ok()? as u32;
    if size == 0 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.get().ok()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).ok()?;
    Some(buf)
}

fn read_info(session: &Session) -> Option<MediaInfo> {
    let props = session.TryGetMediaPropertiesAsync().ok()?.get().ok()?;
    let title = props.Title().ok()?.to_string();
    let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
    let playing = session
        .GetPlaybackInfo()
        .ok()
        .and_then(|i| i.PlaybackStatus().ok())
        .map(|s| s == PlaybackStatus::Playing)
        .unwrap_or(false);
    let art = props.Thumbnail().ok().and_then(read_thumbnail);
    let (b64, mime) = match art {
        Some(bytes) => (Some(base64::engine::general_purpose::STANDARD.encode(&bytes)), sniff_mime(&bytes)),
        None => (None, "image/png"),
    };
    if title.is_empty() && artist.is_empty() {
        return None;
    }
    Some(MediaInfo { title, artist, playing, artwork_png_base64: b64, artwork_mime: mime.into() })
}

fn refresh() {
    let info = MANAGER.get().and_then(|m| m.GetCurrentSession().ok()).and_then(|s| read_info(&s));
    *LAST.lock().unwrap() = info.clone();
    if let Some(app) = APP.get() {
        let _ = app.emit("media-changed", info);
    }
}

fn attach_current() {
    let mut slot = ATTACHED.lock().unwrap();
    if let Some((old, t1, t2)) = slot.take() {
        let _ = old.RemoveMediaPropertiesChanged(t1);
        let _ = old.RemovePlaybackInfoChanged(t2);
    }
    let Some(session) = MANAGER.get().and_then(|m| m.GetCurrentSession().ok()) else { return };
    let h1 = TypedEventHandler::<Session, MediaPropertiesChangedEventArgs>::new(|_, _| {
        refresh();
        Ok(())
    });
    let h2 = TypedEventHandler::<Session, PlaybackInfoChangedEventArgs>::new(|_, _| {
        refresh();
        Ok(())
    });
    if let (Ok(t1), Ok(t2)) = (session.MediaPropertiesChanged(&h1), session.PlaybackInfoChanged(&h2)) {
        *slot = Some((session, t1, t2));
    }
}

#[tauri::command]
pub fn get_media() -> Option<MediaInfo> {
    LAST.lock().unwrap().clone()
}

fn with_session(f: impl FnOnce(&Session)) {
    if let Some(s) = MANAGER.get().and_then(|m| m.GetCurrentSession().ok()) {
        f(&s);
    }
}

#[tauri::command]
pub fn media_toggle() {
    with_session(|s| {
        let _ = s.TryTogglePlayPauseAsync();
    });
}

#[tauri::command]
pub fn media_next() {
    with_session(|s| {
        let _ = s.TrySkipNextAsync();
    });
}

#[tauri::command]
pub fn media_prev() {
    with_session(|s| {
        let _ = s.TrySkipPreviousAsync();
    });
}

/// Event-driven: reacts to session and playback/property changes. No polling.
pub fn start(app: AppHandle) {
    let _ = APP.set(app);
    std::thread::spawn(|| {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let Ok(manager) = Manager::RequestAsync().and_then(|op| op.get()) else { return };
        let _ = MANAGER.set(manager.clone());
        let h = TypedEventHandler::<Manager, CurrentSessionChangedEventArgs>::new(|_, _| {
            attach_current();
            refresh();
            Ok(())
        });
        let _ = manager.CurrentSessionChanged(&h);
        attach_current();
        refresh();
        loop {
            std::thread::park();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_magic_is_png() {
        assert_eq!(sniff_mime(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A]), "image/png");
    }

    #[test]
    fn jpeg_magic_is_jpeg() {
        assert_eq!(sniff_mime(&[0xFF, 0xD8, 0xFF, 0xE0]), "image/jpeg");
    }

    #[test]
    fn unknown_or_empty_defaults_to_png() {
        assert_eq!(sniff_mime(&[]), "image/png");
        assert_eq!(sniff_mime(&[1, 2, 3]), "image/png");
    }
}
