use base64::Engine;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tauri::{AppHandle, Emitter};
use windows::Foundation::TypedEventHandler;
use windows::Media::Control::{
    CurrentSessionChangedEventArgs, GlobalSystemMediaTransportControlsSession as Session,
    SessionsChangedEventArgs,
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
    /// The program that plays it (AppUserModelID, e.g. `MSEdge` or `SpotifyAB.SpotifyMusic_...!Spotify`).
    pub source_app: String,
    /// What the player says it plays: "video", "music", or "" (most do not say: Netflix in Edge and Spotify do not).
    pub playback_type: String,
}

/// The `MediaPlaybackType` of Windows as the word the front end uses ("" for unknown and images).
pub fn playback_type_word(kind: windows::Media::MediaPlaybackType) -> &'static str {
    use windows::Media::MediaPlaybackType;
    if kind == MediaPlaybackType::Video {
        "video"
    } else if kind == MediaPlaybackType::Music {
        "music"
    } else {
        ""
    }
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

pub fn read_thumbnail(r: IRandomAccessStreamReference) -> Option<Vec<u8>> {
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

// ---------------------------------------------------------------------------------------------
// Sources: every program with a media session (Spotify, a browser tab, a video player...)
// ---------------------------------------------------------------------------------------------

/// One program that has a media session, for the source buttons in the player.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MediaSource {
    /// The program's AppUserModelID; what `select_media_source` is given back.
    pub id: String,
    pub name: String,
    pub icon_png_base64: Option<String>,
    pub playing: bool,
    /// The one the player is showing and controlling.
    pub selected: bool,
}

/// A readable name for a media source id when Windows's Apps folder does not know it: well-known programs get
/// their proper name, anything else its last name part (`Vendor.AppName_hash!App` gives `AppName`).
pub fn friendly_name(app_id: &str) -> String {
    let lower = app_id.to_lowercase();
    for (needle, name) in [
        ("spotify", "Spotify"),
        ("msedge", "Microsoft Edge"),
        ("chrome", "Chrome"),
        ("firefox", "Firefox"),
        ("opera", "Opera"),
        ("brave", "Brave"),
        ("vlc", "VLC"),
        ("zunemusic", "Media Player"),
        ("mediaplayer", "Media Player"),
        ("wmplayer", "Windows Media Player"),
        ("applemusic", "Apple Music"),
        ("itunes", "iTunes"),
        ("youtube", "YouTube"),
    ] {
        if lower.contains(needle) {
            return name.to_string();
        }
    }
    let before_bang = app_id.split('!').next().unwrap_or(app_id);
    let family = before_bang.split('_').next().unwrap_or(before_bang);
    let family = family.strip_suffix(".exe").or_else(|| family.strip_suffix(".EXE")).unwrap_or(family);
    family.rsplit('.').next().filter(|s| !s.is_empty()).unwrap_or(app_id).to_string()
}

/// Which listed source the user picked, if it is still there (compared ignoring case); otherwise Windows's own pick.
pub fn pick_source(ids: &[String], selected: &Option<String>) -> Option<usize> {
    let wanted = selected.as_ref()?;
    ids.iter().position(|id| id.eq_ignore_ascii_case(wanted))
}

/// The source the user chose (`None` follows Windows's current session).
static SELECTED: Mutex<Option<String>> = Mutex::new(None);
/// Name and icon per source id, looked up once.
static SOURCE_INFO: Mutex<Option<HashMap<String, (String, Option<String>)>>> = Mutex::new(None);

/// The session the player shows and controls: the chosen source while it still exists, otherwise Windows's current one.
fn chosen_session() -> Option<Session> {
    let manager = MANAGER.get()?;
    let selected = SELECTED.lock().unwrap().clone();
    if selected.is_some() {
        let sessions: Vec<Session> = manager.GetSessions().ok()?.into_iter().collect();
        let ids: Vec<String> = sessions.iter().map(|s| s.SourceAppUserModelId().map(|i| i.to_string()).unwrap_or_default()).collect();
        if let Some(at) = pick_source(&ids, &selected) {
            return sessions.into_iter().nth(at);
        }
        // The chosen program closed: back to following Windows.
        *SELECTED.lock().unwrap() = None;
    }
    manager.GetCurrentSession().ok()
}

fn source_info(id: &str) -> (String, Option<String>) {
    if let Some(known) = SOURCE_INFO.lock().unwrap().as_ref().and_then(|m| m.get(id)).cloned() {
        return known;
    }
    // Windows's Apps folder knows browsers and Store apps by this id: its name and icon are the right ones.
    let info = crate::pinned_apps::store_info(id).map(|(name, icon)| (name, icon)).unwrap_or_else(|| (friendly_name(id), None));
    SOURCE_INFO.lock().unwrap().get_or_insert_with(HashMap::new).insert(id.to_string(), info.clone());
    info
}

fn list_sources() -> Vec<MediaSource> {
    let Some(manager) = MANAGER.get() else { return Vec::new() };
    let Ok(sessions) = manager.GetSessions() else { return Vec::new() };
    let chosen = chosen_session().and_then(|s| s.SourceAppUserModelId().ok()).map(|i| i.to_string());
    let mut out: Vec<MediaSource> = sessions
        .into_iter()
        .filter_map(|session| {
            let id = session.SourceAppUserModelId().ok()?.to_string();
            let playing = session.GetPlaybackInfo().ok().and_then(|i| i.PlaybackStatus().ok()).is_some_and(|s| s == PlaybackStatus::Playing);
            let (name, icon) = source_info(&id);
            let selected = chosen.as_ref().is_some_and(|c| c.eq_ignore_ascii_case(&id));
            Some(MediaSource { id, name, icon_png_base64: icon, playing, selected })
        })
        .collect();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Every program with a media session, for the source buttons (the buttons show only with two or more).
#[tauri::command]
pub async fn get_media_sources() -> Vec<MediaSource> {
    tauri::async_runtime::spawn_blocking(list_sources).await.unwrap_or_default()
}

/// Shows and controls this source (`None`: follow Windows again).
#[tauri::command]
pub fn select_media_source(id: Option<String>) {
    *SELECTED.lock().unwrap() = id;
    std::thread::spawn(|| {
        attach_current();
        refresh();
    });
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
    let source_app = session.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default();
    let playback_type = props.PlaybackType().ok().and_then(|r| r.Value().ok()).map(playback_type_word).unwrap_or("").to_string();
    Some(MediaInfo { title, artist, playing, artwork_png_base64: b64, artwork_mime: mime.into(), position_ms, duration_ms, updated_at_ms, seek_enabled, source_app, playback_type })
}

/// One line in `%APPDATA%\MacDows\logs\media.log` whenever what is playing changes: which program, what kind it says, the title and the cover.
/// The robot tells video from music with these (see `mediaActivity.ts`), and what a player reports is not always what one would expect.
fn log_media(info: &Option<MediaInfo>) {
    use std::io::Write;
    static LAST_LINE: Mutex<String> = Mutex::new(String::new());
    let line = match info {
        Some(i) => format!("source={} type={:?} playing={} title={:?} artist={:?} cover={}", i.source_app, i.playback_type, i.playing, i.title, i.artist, i.artwork_png_base64.as_ref().map_or(0, |b| b.len())),
        None => "nothing playing".to_string(),
    };
    let mut last = LAST_LINE.lock().unwrap_or_else(|e| e.into_inner());
    if *last == line {
        return;
    }
    *last = line.clone();
    let dir = std::env::var_os("APPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir).join("MacDows").join("logs");
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(dir.join("media.log")) {
        let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let _ = writeln!(f, "[{secs}] {line}");
    }
}

fn refresh() {
    let info = chosen_session().and_then(|s| read_info(&s));
    log_media(&info);
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
    let Some(session) = chosen_session() else { return };
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

/// Whether a media session is playing right now.
pub fn is_playing() -> bool {
    LAST.lock().map(|l| l.as_ref().is_some_and(|m| m.playing)).unwrap_or(false)
}

#[tauri::command]
pub fn get_media() -> Option<MediaInfo> {
    LAST.lock().unwrap().clone()
}

fn with_session(f: impl FnOnce(&Session)) {
    if let Some(s) = chosen_session() {
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
        let sessions_changed = TypedEventHandler::<Manager, SessionsChangedEventArgs>::new(|_, _| {
            attach_current();
            refresh();
            Ok(())
        });
        let _ = manager.SessionsChanged(&sessions_changed);
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

    #[test]
    fn well_known_programs_get_their_proper_names() {
        assert_eq!(friendly_name("SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify"), "Spotify");
        assert_eq!(friendly_name("Spotify.exe"), "Spotify");
        assert_eq!(friendly_name("MSEdge"), "Microsoft Edge");
        assert_eq!(friendly_name("chrome.exe"), "Chrome");
        assert_eq!(friendly_name("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"), "Media Player");
    }

    #[test]
    fn an_unknown_program_is_named_by_the_last_part_of_its_id() {
        assert_eq!(friendly_name("Vendor.CoolPlayer_abc123!App"), "CoolPlayer");
        assert_eq!(friendly_name("foo.exe"), "foo");
        assert_eq!(friendly_name("Plain"), "Plain");
    }

    #[test]
    fn the_chosen_source_is_found_ignoring_case_and_a_missing_one_is_not() {
        let ids = vec!["MSEdge".to_string(), "Spotify.exe".to_string()];
        assert_eq!(pick_source(&ids, &Some("spotify.EXE".to_string())), Some(1));
        assert_eq!(pick_source(&ids, &Some("Gone".to_string())), None);
        assert_eq!(pick_source(&ids, &None), None);
        assert_eq!(pick_source(&[], &Some("MSEdge".to_string())), None);
    }
}
