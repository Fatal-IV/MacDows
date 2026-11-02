import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Visibility = "fixed" | "smart" | "peek";
export interface Settings {
  language: string;
  notch_visibility: Visibility;
  dock_visibility: Visibility;
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
}

export const getMedia = () => invoke<MediaInfo | null>("get_media");
export const mediaToggle = () => invoke<void>("media_toggle");
export const mediaNext = () => invoke<void>("media_next");
export const mediaPrev = () => invoke<void>("media_prev");
export const onMediaChanged = (cb: (m: MediaInfo | null) => void): Promise<UnlistenFn> =>
  listen<MediaInfo | null>("media-changed", (e) => cb(e.payload));

export const onSettingsChanged = (cb: (s: Settings) => void): Promise<UnlistenFn> =>
  listen<Settings>("settings-changed", (e) => cb(e.payload));
