import type { Island, ScreenId } from "../bridge";

/** The screens in their default order (`SCREEN_IDS` in `island_settings.rs`). Only a type is imported from the
 *  bridge, so this module and its tests run without the Tauri API. */
export const DEFAULT_ORDER: ScreenId[] = ["status", "music", "tools", "quick", "notifications", "shelf"];

/** The island settings as they are when nothing was changed. Mirrors `Island::default()` in `island_settings.rs`. */
export function defaultIsland(): Island {
  return {
    visibility: "fixed",
    fullscreen: "edge",
    nav: { wheel: true, order: [...DEFAULT_ORDER], click: "smart", middle: true, remember: false },
    screens: {
      status: { enabled: true, clock_12h: false, battery_percent: true },
      music: { enabled: true, visualizer: true, album_glow: true, progress: true, auto_open: true, artwork: true },
      tools: { enabled: true, calendar: true, week_starts_monday: true, timer: true, timer_sound: true, timer_progress: true, stopwatch: true },
      quick: { enabled: true, wifi: true, bluetooth: true, dnd: true, battery_saver: true, airplane: true, hotspot: true, accessibility: true, brightness: true, volume: true, lock: true, macdows_settings: true, windows_settings: true },
      notifications: { enabled: true, banner: true, banner_seconds: 5, save_history: true, history_max: 100, muted_apps: [] },
      shelf: { enabled: true, persist: false, max_items: 40, auto_open: true },
    },
  };
}
