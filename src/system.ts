import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";

export type ShellAction = "notifications" | "quick_settings" | "hidden_icons";

export interface SystemStatus {
  battery_percent: number | null;
  hidden_icons_available: boolean;
  taskbar_hidden: boolean;
}

export const shellAction = (action: ShellAction) => invoke<void>("shell_action", { action });
export const getSystemStatus = () => invoke<SystemStatus>("get_system_status");

/** Battery and taskbar state, refreshed every 20 seconds (the battery has no change event here). */
export function useSystemStatus(): SystemStatus | null {
  const [status, setStatus] = useState<SystemStatus | null>(null);
  useEffect(() => {
    const refresh = () => getSystemStatus().then(setStatus).catch(() => {});
    refresh();
    const id = setInterval(refresh, 20_000);
    return () => clearInterval(id);
  }, []);
  return status;
}
