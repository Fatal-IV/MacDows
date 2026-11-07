/** How the robot watches a video: eating popcorn, or looking at a screen that shows how far the video is. Chosen in Settings. */
export type WatchStyle = "popcorn" | "progress";

export const WATCH_STYLE_KEY = "macdows-watch-style";

export function readWatchStyle(stored: string | null): WatchStyle {
  return stored === "progress" ? "progress" : "popcorn";
}

/** How far the video is, 0 to 1: the position when it was measured plus the time since then while it plays. 0 when the length is unknown. */
export function watchProgress(positionMs: number, durationMs: number, updatedAtMs: number, playing: boolean, nowMs: number): number {
  if (!(durationMs > 0)) return 0;
  const elapsed = playing && updatedAtMs > 0 ? Math.max(0, nowMs - updatedAtMs) : 0;
  return Math.min(1, Math.max(0, (positionMs + elapsed) / durationMs));
}
