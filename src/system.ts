import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export type ShellAction = "notifications" | "quick_settings" | "hidden_icons" | "start" | "search" | "show_desktop";

export interface SystemStatus {
  battery_percent: number | null;
  battery_charging: boolean;
  hidden_icons_available: boolean;
  taskbar_hidden: boolean;
}

export const shellAction = (action: ShellAction) => invoke<void>("shell_action", { action });
export const getSystemStatus = () => invoke<SystemStatus>("get_system_status");

/** Battery and taskbar state: refreshed on `taskbar-changed` (hide/restore finished) and every 20 seconds for the battery. */
export function useSystemStatus(): SystemStatus | null {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  useEffect(() => {
    const refresh = () => getSystemStatus().then(setStatus).catch(() => {});
    refresh();
    const id = setInterval(refresh, 20_000);
    // At startup the backend may hide the taskbar (and emit `taskbar-changed`) before this listener exists: look every second until it is hidden.
    let tries = 0;
    const fast = setInterval(() => {
      tries += 1;
      getSystemStatus()
        .then((s) => {
          setStatus(s);
          if (s.taskbar_hidden || tries >= 30) clearInterval(fast);
        })
        .catch(() => {});
    }, 1000);
    const un = listen("taskbar-changed", refresh);
    return () => {
      clearInterval(id);
      clearInterval(fast);
      un.then((f) => f());
    };
  }, []);
  return status;
}
