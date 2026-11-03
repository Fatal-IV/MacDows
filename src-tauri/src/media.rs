use base64::Engine;
use serde::Serialize;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession as Session,
    GlobalSystemMediaTransportControlsSessionManager as Manager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as PlaybackStatus, MediaPropertiesChangedEventArgs,
    PlaybackInfoChangedEventArgs, TimelinePropertiesChangedEventArgs,
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
    /// Where the track is, in milliseconds, as of `updated_at_ms`. While playing, add the time since then.
    pub position_ms: u64,
    pub duration_ms: u64,
    /// Unix time in milliseconds when `position_ms` was measured.
    pub updated_at_ms: u64,
    /// Whether the player accepts a seek.
    pub seek_enabled: bool,
}

const TICKS_PER_MS: i64 = 10_000;
/// Milliseconds between 1601-01-01 (Windows time) and 1970-01-01.
const WINDOWS_TO_UNIX_MS: i64 = 11_644_473_600_000;

/// Position and length of the track in milliseconds, from the player's start/end/position (all in 100 ns ticks).
pub fn timeline_ms(position: i64, start: i64, end: i64) -> (u64, u64) {
    let duration = ((end - start) / TICKS_PER_MS).max(0) as u64;
    let position = ((position - start) / TICKS_PER_MS).max(0) as u64;
    (if duration > 0 { position.min(duration) } else { position }, duration)
}

/// Windows `DateTime` ticks (100 ns since 1601) as Unix milliseconds.
pub fn windows_ticks_to_unix_ms(ticks: i64) -> u64 {
    (ticks / TICKS_PER_MS - WINDOWS_TO_UNIX_MS).max(0) as u64
}

/// The player's timeline position in ticks for a seek to `position_ms`, never past the end.
pub fn seek_ticks(position_ms: u64, start: i64, end: i64) -> i64 {
    let wanted = start.saturating_add((position_ms as i64).saturating_mul(TICKS_PER_MS));
    if end > start { wanted.min(end) } else { wanted }
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
static ATTACHED: Mutex<Option<(Session, i64, i64, i64)>> = Mutex::new(None);

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
    let (position_ms, duration_ms, updated_at_ms) = session
        .GetTimelineProperties()
        .ok()
        .map(|t| {
            let ticks = |r: windows::core::Result<windows::Foundation::TimeSpan>| r.map(|d| d.Duration).unwrap_or(0);
            let (pos, dur) = timeline_ms(ticks(t.Position()), ticks(t.StartTime()), ticks(t.EndTime()));
            let updated = t.LastUpdatedTime().map(|d| windows_ticks_to_unix_ms(d.UniversalTime)).unwrap_or(0);
            (pos, dur, updated)
        })
        .unwrap_or((0, 0, 0));
    let seek_enabled = session
        .GetPlaybackInfo()
        .ok()
        .and_then(|i| i.Controls().ok())
        .and_then(|c| c.IsPlaybackPositionEnabled().ok())
        .unwrap_or(false);
    Some(MediaInfo { title, artist, playing, artwork_png_base64: b64, artwork_mime: mime.into(), position_ms, duration_ms, updated_at_ms, seek_enabled })
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
    if let Some((old, t1, t2, t3)) = slot.take() {
        let _ = old.RemoveMediaPropertiesChanged(t1);
        let _ = old.RemovePlaybackInfoChanged(t2);
        let _ = old.RemoveTimelinePropertiesChanged(t3);
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
    // A seek or a track change moves the timeline; the position is re-read then (it is not polled).
    let h3 = TypedEventHandler::<Session, TimelinePropertiesChangedEventArgs>::new(|_, _| {
        refresh();
        Ok(())
    });
    if let (Ok(t1), Ok(t2), Ok(t3)) = (session.MediaPropertiesChanged(&h1), session.PlaybackInfoChanged(&h2), session.TimelinePropertiesChanged(&h3)) {
        *slot = Some((session, t1, t2, t3));
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

/// Jumps to `position_ms` in the current track.
#[tauri::command]
pub fn media_seek(position_ms: u64) {
    with_session(|s| {
        let Ok(t) = s.GetTimelineProperties() else { return };
        let ticks = |r: windows::core::Result<windows::Foundation::TimeSpan>| r.map(|d| d.Duration).unwrap_or(0);
        let target = seek_ticks(position_ms, ticks(t.StartTime()), ticks(t.EndTime()));
        let _ = s.TryChangePlaybackPositionAsync(target);
    });
    // Some players (the Windows Media Player among them) send no timeline event after a seek, so read it again.
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(250));
        refresh();
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
    fn the_timeline_is_measured_from_the_start_time_in_milliseconds() {
        // 30 s into a track that runs from 0 to 3 min 15 s.
        assert_eq!(timeline_ms(300_000_000, 0, 1_950_000_000), (30_000, 195_000));
        // A player whose timeline starts at 10 s.
        assert_eq!(timeline_ms(450_000_000, 100_000_000, 1_100_000_000), (35_000, 100_000));
    }

    #[test]
    fn the_position_never_passes_the_end_or_goes_negative() {
        assert_eq!(timeline_ms(9_000_000_000, 0, 1_000_000_000), (100_000, 100_000));
        assert_eq!(timeline_ms(-5, 0, 1_000_000_000), (0, 100_000));
        // A live stream with no length keeps its raw position.
        assert_eq!(timeline_ms(50_000_000, 0, 0), (5_000, 0));
    }

    #[test]
    fn windows_time_is_converted_to_unix_milliseconds() {
        // 1970-01-01 in Windows ticks.
        assert_eq!(windows_ticks_to_unix_ms(116_444_736_000_000_000), 0);
        assert_eq!(windows_ticks_to_unix_ms(116_444_736_000_000_000 + 10_000 * 1_500), 1_500);
        assert_eq!(windows_ticks_to_unix_ms(0), 0);
    }

    #[test]
    fn a_seek_is_placed_after_the_start_time_and_capped_at_the_end() {
        assert_eq!(seek_ticks(30_000, 0, 1_950_000_000), 300_000_000);
        assert_eq!(seek_ticks(30_000, 100_000_000, 1_100_000_000), 400_000_000);
        assert_eq!(seek_ticks(999_999, 0, 1_000_000_000), 1_000_000_000);
    }

    #[test]
    fn unknown_or_empty_defaults_to_png() {
        assert_eq!(sniff_mime(&[]), "image/png");
        assert_eq!(sniff_mime(&[1, 2, 3]), "image/png");
    }
}
