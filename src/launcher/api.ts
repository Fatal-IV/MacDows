import { invoke } from "@tauri-apps/api/core";

export interface AppEntry {
  id: string;
  name: string;
  icon_png_base64: string | null;
}

export interface StartState {
  pinned: string[];
  recent: string[];
  user: string;
}

export const listApps = () => invoke<AppEntry[]>("launcher_apps");
export const startState = () => invoke<StartState>("launcher_state");
export const launchApp = (id: string) => invoke<boolean>("launch_app", { id });
export const togglePin = (id: string) => invoke<void>("toggle_pin", { id });
export const openLocation = (kind: string) => invoke<boolean>("open_location", { kind });
export const powerAction = (kind: string) => invoke<boolean>("power_action", { kind });
export const closePopup = (label: string) => invoke<void>("close_popup", { label }).catch(() => {});

export const iconSrc = (app: { icon_png_base64: string | null }) => (app.icon_png_base64 ? `data:image/png;base64,${app.icon_png_base64}` : null);
