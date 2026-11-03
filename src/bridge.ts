import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Visibility = "fixed" | "smart" | "peek";
export interface Settings {
  language: string;
  notch_visibility: Visibility;
  dock_visibility: Visibility;
  check_updates: boolean;
  last_update_check: number | null;
  replace_taskbar: boolean;
  taskbar_notice_shown: boolean;
  start_with_windows: boolean;
  show_desktop_button: boolean;
}

export const getSettings = () => invoke<Settings>("get_settings");
export const setSetting = (key: keyof Settings, value: unknown) =>
  invoke<Settings>("set_setting", { key, value });

export const onFullscreenChanged = (cb: (fullscreen: boolean) => void): Promise<UnlistenFn> =>
  listen<boolean>("fullscreen-changed", (e) => cb(e.payload));

export interface AppWindow {
  id: number;
  title: string;
  exe: string;
  icon_png_base64: string | null;
}

export const listWindows = () => invoke<AppWindow[]>("list_windows");
export const focusWindow = (id: number) => invoke<void>("focus_window", { id });
export const onWindowsChanged = (cb: () => void): Promise<UnlistenFn> => listen("windows-changed", cb);

export interface MediaInfo {
  title: string;
  artist: string;
  playing: boolean;
  artwork_png_base64: string | null;
  artwork_mime: string;
  /** Position at `updated_at_ms`; while playing, add the time since then. */
  position_ms: number;
  duration_ms: number;
  updated_at_ms: number;
  seek_enabled: boolean;
}

export interface Volume {
  level: number;
  muted: boolean;
}

export const getMedia = () => invoke<MediaInfo | null>("get_media");
export const mediaToggle = () => invoke<void>("media_toggle");
export const mediaNext = () => invoke<void>("media_next");
export const mediaPrev = () => invoke<void>("media_prev");
export const mediaSeek = (positionMs: number) => invoke<void>("media_seek", { positionMs });
export interface QuickTile {
  available: boolean;
  on: boolean;
}
export interface QuickSettings {
  wifi: QuickTile;
  bluetooth: QuickTile;
  dnd: QuickTile;
  battery_saver: QuickTile;
}
export type QuickKey = keyof QuickSettings;
export const getQuickSettings = () => invoke<QuickSettings>("get_quick_settings");
export const setQuickSetting = (key: QuickKey, on: boolean) => invoke<QuickSettings>("set_quick_setting", { key, on });

export type QuickShortcut = "screenshot" | "settings" | "task_manager" | "calculator" | "lock";
export const openMacdowsSettings = () => invoke<void>("open_macdows_settings");
export const quickLaunch = (id: QuickShortcut) => invoke<void>("quick_launch", { id });

export const getBrightness = () => invoke<number | null>("get_brightness");
export const setBrightness = (level: number) => invoke<void>("set_brightness", { level });

export const getVolume = () => invoke<Volume | null>("get_volume");
export const setVolume = (level: number) => invoke<Volume | null>("set_volume", { level });
export const onMediaChanged = (cb: (m: MediaInfo | null) => void): Promise<UnlistenFn> =>
  listen<MediaInfo | null>("media-changed", (e) => cb(e.payload));

export const onSettingsChanged = (cb: (s: Settings) => void): Promise<UnlistenFn> =>
  listen<Settings>("settings-changed", (e) => cb(e.payload));

export interface Notice {
  id: number;
  app: string;
  title: string;
  body: string;
  /** Unix time in milliseconds. */
  time_ms: number;
  icon_png_base64: string | null;
}

export const getNotifications = () => invoke<Notice[]>("get_notifications");
export const removeNotification = (id: number) => invoke<void>("remove_notification", { id });
export const clearNotifications = () => invoke<void>("clear_notifications");
export const onNotificationNew = (cb: (n: Notice) => void): Promise<UnlistenFn> => listen<Notice>("notification-new", (e) => cb(e.payload));
export const onNotificationsChanged = (cb: () => void): Promise<UnlistenFn> => listen("notifications-changed", cb);

export interface ShelfItem {
  path: string;
  name: string;
  is_dir: boolean;
  missing: boolean;
  icon_png_base64: string | null;
  icon_path: string | null;
}

export const getShelf = () => invoke<ShelfItem[]>("get_shelf");
export const addToShelf = (paths: string[]) => invoke<ShelfItem[]>("add_to_shelf", { paths });
export const removeFromShelf = (path: string) => invoke<ShelfItem[]>("remove_from_shelf", { path });
export const clearShelf = () => invoke<void>("clear_shelf");
export const openShelfItem = (path: string) => invoke<void>("open_shelf_item", { path });
export const revealShelfItem = (path: string) => invoke<void>("reveal_shelf_item", { path });
