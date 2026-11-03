import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Lang } from "./i18n";

export type Decision = "fresh" | "reinstall" | "older" | "newer";
export interface InstallInfo {
  decision: Decision;
  installed_version: string | null;
  bundled_version: string;
  lang: Lang;
  log_path: string;
}
export interface StageEvent {
  stage: "preparing" | "installing" | "finishing" | "done";
  percent: number;
}

export const inTauri = "__TAURI_INTERNALS__" in window;
const query = new URLSearchParams(window.location.search);

/** Screens can be forced in the browser with ?screen=ready|working|done|error|installed|newer&lang=tr|en. */
export const mockScreen = inTauri ? null : query.get("screen");

const mockInfo = (): InstallInfo => {
  const screen = query.get("screen");
  const decision: Decision = screen === "installed" ? "reinstall" : screen === "newer" ? "newer" : "fresh";
  return {
    decision,
    installed_version: decision === "fresh" ? null : "0.1.0",
    bundled_version: "0.1.0",
    lang: query.get("lang") === "tr" ? "tr" : "en",
    log_path: String.raw`C:\Users\you\AppData\Local\MacDows-Setup\setup.log`,
  };
};

export const installState = () => (inTauri ? invoke<InstallInfo>("install_state") : Promise.resolve(mockInfo()));
/** `autostart` is only given for a fresh install; leaving it out keeps what an existing install has. */
export const startInstall = (autostart?: boolean) => (inTauri ? invoke<void>("start_install", { autostart: autostart ?? null }) : Promise.resolve());
export const launchApp = () => (inTauri ? invoke<void>("launch_app") : Promise.resolve());
export const closeWindow = () => (inTauri ? getCurrentWindow().close() : Promise.resolve());

const noop: Promise<UnlistenFn> = Promise.resolve(() => {});
export const onStage = (cb: (e: StageEvent) => void) => (inTauri ? listen<StageEvent>("install-stage", (e) => cb(e.payload)) : noop);
export const onDone = (cb: () => void) => (inTauri ? listen("install-done", () => cb()) : noop);
export const onError = (cb: (message: string) => void) =>
  inTauri ? listen<{ message: string }>("install-error", (e) => cb(e.payload.message)) : noop;
