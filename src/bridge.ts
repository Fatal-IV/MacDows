import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Visibility = "fixed" | "smart" | "peek";
export interface LocalPin {
  id: string;
  name: string;
  exe: string;
  aumid: string;
}
/** What MacDows adds on top of the pinned apps Windows lists. */
export interface Dock {
  pinned_order: string[];
  local_pins: LocalPin[];
  /** Pin id (or exe path) to the file name of its custom icon. */
  custom_icons: Record<string, string>;
  show_drives: boolean;
  previews: boolean;
  win_number: boolean;
}

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
  show_pinned_apps: boolean;
  /** Pins hidden in the dock because Windows would not unpin them. */
  hidden_pinned: string[];
  dock: Dock;
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
  /** AppUserModelID, "" when unknown. */
  aumid: string;
  icon_png_base64: string | null;
}

export const listWindows = () => invoke<AppWindow[]>("list_windows");
export const focusWindow = (id: number) => invoke<void>("focus_window", { id });
export interface PinnedApp {
  /** The shortcut's full path; pass it to `launchPinned`. */
  id: string;
  name: string;
  exe: string;
  /** AppUserModelID of a Store app, "" otherwise. */
  aumid: string;
  icon_png_base64: string | null;
}
export const listPinned = () => invoke<PinnedApp[]>("list_pinned");
export const launchPinned = (id: string) => invoke<void>("launch_pinned", { id });
/** What the dock tells Rust about the icon (or empty dock area) that was right-clicked. */
export interface DockMenuTarget {
  kind: "pinned" | "running" | "empty";
  id: string;
  exe: string;
  aumid: string;
  name: string;
  hwnds: number[];
}
/** Shows the dock's right-click menu at the pointer. */
export const dockMenu = (target: DockMenuTarget) => invoke<void>("dock_menu", { target });
export interface Drive {
  /** "C" (no colon). */
  letter: string;
  kind: "local" | "removable";
  /** The volume's name, "" when it has none. */
  label: string;
}
export const listDrives = () => invoke<Drive[]>("list_drives");
export const openDrive = (letter: string) => invoke<void>("open_drive", { letter });
export const onDrivesChanged = (cb: () => void): Promise<UnlistenFn> => listen("drives-changed", cb);
/** The dock tells Rust how many pinned apps it shows, so Win+number leaves empty slots to Windows. */
export const setDockPinnedCount = (count: number) => invoke<void>("set_dock_pinned_count", { count });
export const launchNewApp = (id: string, exe: string, aumid: string) => invoke<void>("launch_new_app", { id, exe, aumid });
/** Win+number (slot 0 to 8); `new_instance` when Shift was held too. */
export const onDockShortcut = (cb: (s: { index: number; new_instance: boolean }) => void): Promise<UnlistenFn> =>
  listen<{ index: number; new_instance: boolean }>("dock-shortcut", (e) => cb(e.payload));
export interface PreviewItem {
  hwnd: number;
  title: string;
  /** A PNG of the window, null when it cannot be captured. */
  thumbnail: string | null;
  /** The icon of the app, the fallback picture. */
  icon: string | null;
}
/** Opens the window previews above an icon. `centerX` and `dockTop` are CSS pixels in the dock window. */
export const showPreview = (centerX: number, dockTop: number, icon: string | null, hwnds: number[]) => invoke<void>("show_preview", { centerX, dockTop, icon, hwnds });
export const previewHover = (source: "dock" | "panel", inside: boolean) => invoke<void>("preview_hover", { source, inside });
export const previewItems = () => invoke<PreviewItem[]>("preview_items");
export const closePreview = () => invoke<void>("close_preview");
export const previewActivate = (hwnd: number) => invoke<void>("preview_activate", { hwnd });
export const previewCloseWindow = (hwnd: number) => invoke<void>("preview_close_window", { hwnd });
export const onPreviewUpdated = (cb: () => void): Promise<UnlistenFn> => listen("preview-updated", cb);
/** Custom icons (PNG, base64) by lowercase key: a pin's id, or for other apps their program path. */
export const getCustomIcons = () => invoke<Record<string, string>>("get_custom_icons");
/** Saves the order the pinned apps were dragged into (their ids, left to right). */
export const setPinnedOrder = (ids: string[]) => invoke<void>("set_pinned_order", { ids });
export const onPinnedChanged = (cb: () => void): Promise<UnlistenFn> => listen("pinned-changed", cb);
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
  airplane: QuickTile;
  hotspot: QuickTile;
}
export type QuickKey = keyof QuickSettings;
export const getQuickSettings = () => invoke<QuickSettings>("get_quick_settings");
export const setQuickSetting = (key: QuickKey, on: boolean) => invoke<QuickSettings>("set_quick_setting", { key, on });

export type QuickShortcut = "screenshot" | "settings" | "task_manager" | "calculator" | "lock" | "bluetooth_settings" | "wifi_settings" | "accessibility";
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

export const getNotificationHistory = () => invoke<Notice[]>("get_notification_history");
export const clearNotificationHistory = () => invoke<void>("clear_notification_history");
export const onNotificationHistoryChanged = (cb: () => void): Promise<UnlistenFn> => listen("notification-history-changed", cb);

export type BtKind = "audio" | "phone" | "computer" | "input" | "other";
export interface BtDevice {
  id: string;
  name: string;
  address: string;
  connected: boolean;
  battery: number | null;
  kind: BtKind;
}
export const getBluetoothDevices = () => invoke<BtDevice[]>("get_bluetooth_devices");
export const setBluetoothConnection = (address: string, connect: boolean) => invoke<BtDevice[]>("set_bluetooth_connection", { address, connect });
