/** Video or music (see also `log_media` in `media.rs`, which records what the players report): the robot sits in an armchair in front of a screen for video and shakes its head for music. Windows is not much help
 *  (Netflix in Edge reports only the page title "Netflix": no artist, no cover and no kind), so the source and the shape of the cover count. */
export type Activity = "video" | "music";

export interface ActivityInput {
  source_app: string;
  /** "video", "music" or "" when the player does not say. */
  playback_type: string;
  title: string;
  artist: string;
}

const BROWSER = /msedge|chrome|firefox|opera|brave|vivaldi|iexplore/i;
const VIDEO_PLAYER = /vlc|mpc-|mpc_|potplayer|zunevideo|netflix|disney|primevideo|amazonvideo|kodi|plex|mpv|wmplayer/i;
const STREAMING_SITE = /netflix|prime video|disney|hbo|hulu|tabii|blutv|exxen|twitch/i;

/** `artAspect` is the width over the height of the cover, or null when there is none. Album covers are square, video thumbnails wide. */
export function mediaActivity(m: ActivityInput, artAspect: number | null): Activity {
  // Only "video" is believed: Edge reports "music" for everything, Netflix included (seen in media.log), and Spotify says nothing.
  if (m.playback_type === "video") return "video";
  if (VIDEO_PLAYER.test(m.source_app)) return "video";
  if (!BROWSER.test(m.source_app)) return "music";
  if (STREAMING_SITE.test(m.title)) return "video";
  if (artAspect !== null) return artAspect >= 1.3 ? "video" : "music";
  return m.artist.trim() === "" ? "video" : "music";
}
