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

export type FullscreenMode = "edge" | "hidden";
export type ClickMode = "smart" | "quick" | "player";
export type ScreenId = "status" | "music" | "tools" | "quick" | "notifications" | "shelf";
export const SCREEN_IDS: ScreenId[] = ["status", "music", "tools", "quick", "notifications", "shelf"];

export interface Widgets {
  left: string[];
  right: string[];
}
export interface StatusCfg {
  enabled: boolean;
  clock_12h: boolean;
  widgets: Widgets;
  weather_city: string;
  weather_lat: number;
  weather_lon: number;
  temp_unit: "celsius" | "fahrenheit";
  low_battery: number;
}
export interface MusicCfg {
  enabled: boolean;
  visualizer: boolean;
  album_glow: boolean;
  progress: boolean;
  auto_open: boolean;
  artwork: boolean;
  /** Buttons to switch between programs that play media (shown from two programs). */
  sources: boolean;
}
export interface ToolsCfg {
  enabled: boolean;
  calendar: boolean;
  week_starts_monday: boolean;
  timer: boolean;
  timer_sound: boolean;
  timer_progress: boolean;
  stopwatch: boolean;
}
export interface QuickCfg {
  enabled: boolean;
  wifi: boolean;
  bluetooth: boolean;
  dnd: boolean;
  battery_saver: boolean;
  airplane: boolean;
  hotspot: boolean;
  accessibility: boolean;
  brightness: boolean;
  volume: boolean;
  lock: boolean;
  macdows_settings: boolean;
  windows_settings: boolean;
  /** The Wi-Fi page behind the Wi-Fi tile's arrow. */
  wifi_page: boolean;
  /** The VPN card and page. */
  vpn: boolean;
  /** Pairing new Bluetooth devices inside the island. */
  bluetooth_pairing: boolean;
}
export interface NotifCfg {
  enabled: boolean;
  banner: boolean;
  /** 3 to 10. */
  banner_seconds: number;
  save_history: boolean;
  /** 25, 50 or 100. */
  history_max: number;
  muted_apps: string[];
}
export interface ShelfCfg {
  enabled: boolean;
  persist: boolean;
  max_items: number;
  auto_open: boolean;
}
export interface Screens {
  status: StatusCfg;
  music: MusicCfg;
  tools: ToolsCfg;
  quick: QuickCfg;
  notifications: NotifCfg;
  shelf: ShelfCfg;
}
export interface Nav {
  wheel: boolean;
  /** Every screen id once. */
  order: ScreenId[];
  click: ClickMode;
  middle: boolean;
  remember: boolean;
  /** While media exists, hovering the island opens the player by itself. */
  hover_player: boolean;
}
/** The Dynamic Island: its screens, what each offers, how it is navigated. */
export interface Island {
  visibility: Visibility;
  fullscreen: FullscreenMode;
  nav: Nav;
  screens: Screens;
}

export interface Settings {
  language: string;
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
  island: Island;
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
export interface CatalogApp {
  id: string;
  name: string;
  exe: string;
  aumid: string;
  icon_png_base64: string | null;
  running: boolean;
  pinned: boolean;
}
export const getCatalog = () => invoke<CatalogApp[]>("get_catalog");
export const pinCatalogApp = (id: string) => invoke<void>("pin_catalog_app", { id });
export const closeAddAppPanel = () => invoke<void>("close_add_app_panel");
/** A program with a media session: the buttons in the player switch between these. */
export interface MediaSource {
  id: string;
  name: string;
  icon_png_base64: string | null;
  playing: boolean;
  /** The one the player shows and controls. */
  selected: boolean;
}
export const getMediaSources = () => invoke<MediaSource[]>("get_media_sources");
/** Shows and controls this program; null follows Windows again. */
export const selectMediaSource = (id: string | null) => invoke<void>("select_media_source", { id });
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

export type QuickShortcut = "screenshot" | "settings" | "task_manager" | "calculator" | "lock" | "bluetooth_settings" | "wifi_settings" | "vpn_settings" | "accessibility";
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

export interface WifiNetwork {
  ssid: string;
  /** 0-100 */
  signal: number;
  secured: boolean;
  enterprise: boolean;
  saved: boolean;
  connected: boolean;
}
export interface WifiState {
  available: boolean;
  enabled: boolean;
  networks: WifiNetwork[];
}
export const getWifiState = () => invoke<WifiState>("wifi_state");
export const wifiScan = () => invoke<void>("wifi_scan");
export const wifiConnect = (ssid: string) => invoke<void>("wifi_connect", { ssid });
export const wifiConnectNew = (ssid: string, password: string | null, auth: "open" | "wpa2" | "wpa3", hidden: boolean) => invoke<void>("wifi_connect_new", { ssid, password, auth, hidden });
export const islandFocus = (on: boolean) => invoke<void>("island_focus", { on });
export const wifiDisconnect = () => invoke<void>("wifi_disconnect");
export const wifiForget = (ssid: string) => invoke<void>("wifi_forget", { ssid });

export interface VpnEntry {
  name: string;
  state: "connected" | "disconnected" | "connecting" | string;
}
export interface VpnState {
  entries: VpnEntry[];
  app_vpn_active: boolean;
}
export const getVpnState = () => invoke<VpnState>("vpn_state");
export const vpnConnect = (name: string) => invoke<void>("vpn_connect", { name });
export const vpnDisconnect = (name: string) => invoke<void>("vpn_disconnect", { name });

export interface BtPairingRequest {
  id: string;
  /** "confirm" | "confirm_pin" | "provide_pin" | "none" */
  kind: string;
  pin: string | null;
}
export const btScanStart = () => invoke<void>("bt_scan_start");
export const btScanStop = () => invoke<void>("bt_scan_stop");
export const btUnpair = (id: string) => invoke<void>("bt_unpair", { id });
export const btPair = (id: string) => invoke<void>("bt_pair", { id });
export const btPairingAnswer = (accept: boolean, pin: string | null) => invoke<void>("bt_pairing_answer", { accept, pin });
export const onBtFound = (cb: (d: { id: string; name: string; kind: string }) => void): Promise<UnlistenFn> => listen<{ id: string; name: string; kind: string }>("bt-found", (e) => cb(e.payload));
export const onBtLost = (cb: (d: { id: string }) => void): Promise<UnlistenFn> => listen<{ id: string }>("bt-lost", (e) => cb(e.payload));
export const onBtPaired = (cb: (d: { id: string }) => void): Promise<UnlistenFn> => listen<{ id: string }>("bt-paired", (e) => cb(e.payload));
export const onBtPairingRequest = (cb: (r: BtPairingRequest) => void): Promise<UnlistenFn> => listen<BtPairingRequest>("bt-pairing-request", (e) => cb(e.payload));
