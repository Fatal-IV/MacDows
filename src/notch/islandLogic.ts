// Which screens of the Dynamic Island can be reached, in what order, and where a click goes. Pure functions of the
// island settings, so the island component only asks questions.
import type { Island, ScreenId } from "../bridge";

/** One place the wheel can stop: the two pill states, the player, and the other screens. */
export type Stop = "status" | "music" | "player" | "tools" | "quick" | "history" | "shelf";

export const screenOn = (island: Island, id: ScreenId): boolean => island.screens[id].enabled;

/** The stops of the wheel in the user's order, leaving out screens that are off. The music screen gives the music
 *  pill and the player, and only while media exists; the notifications screen is the history page. */
export function buildStops(island: Island, hasMedia: boolean): Stop[] {
  const out: Stop[] = [];
  for (const id of island.nav.order) {
    if (!screenOn(island, id)) continue;
    if (id === "music") {
      if (hasMedia) out.push("music", "player");
    } else if (id === "notifications") {
      out.push("history");
    } else {
      out.push(id);
    }
  }
  return out;
}

/** Where a click on the island goes, or null when there is nowhere to go. */
export function clickTarget(island: Island, hasMedia: boolean, playing: boolean): Stop | null {
  const stops = buildStops(island, hasMedia);
  const has = (s: Stop) => stops.includes(s);
  const mode = island.nav.click;
  const wanted: Stop = mode === "player" ? "player" : mode === "quick" ? "quick" : playing && has("player") ? "player" : "quick";
  if (has(wanted)) return wanted;
  // The player has no media behind it: the quick settings are the natural place.
  if (wanted === "player" && has("quick")) return "quick";
  // Otherwise the next screen that is on (the pill states are not places to open).
  return stops.find((s) => s !== "status" && s !== "music") ?? null;
}

/** The pill the island rests on: music while media plays (and that screen is on), otherwise the status pill. */
export function restingStop(island: Island, hasMedia: boolean, playing: boolean): "status" | "music" {
  return hasMedia && playing && screenOn(island, "music") ? "music" : "status";
}

/** The last screen that is on cannot be switched off; anything else can. */
export function canDisable(island: Island, id: ScreenId): boolean {
  if (!screenOn(island, id)) return true;
  return (Object.keys(island.screens) as ScreenId[]).filter((s) => screenOn(island, s)).length > 1;
}

/** Notifications of this app get no banner (compared ignoring case). */
export function mutedApp(island: Island, app: string): boolean {
  const name = app.toLowerCase();
  return island.screens.notifications.muted_apps.some((m) => m.toLowerCase() === name);
}

/** Where the island remembers the app version it has already greeted. */
export const GREETED_KEY = "macdows.greeted";
/** How long the island stays open for its greeting. */
export const GREETING_MS = 3200;

/** The island waves on every start of the app (so also after the computer has booted and the app starts with Windows),
 *  not only on the first start of a version. */
export const GREET_EVERY_START = true;

/** How long the island waits before its next random wave: 45 s to 150 s (`rand` is 0..1). */
export function nextWaveDelay(rand: number): number {
  return Math.round(45_000 + Math.min(1, Math.max(0, rand)) * 105_000);
}

/** The island greets on the first start of a version: a fresh install (nothing stored) or an update (another version
 *  stored). `always` (see GREET_EVERY_START) makes it greet on every start. */
export function shouldGreet(stored: string | null, version: string, always = false): boolean {
  return version !== "" && (always || stored !== version);
}
