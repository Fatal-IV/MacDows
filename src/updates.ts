import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";

export type UpdateStatus =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "up_to_date"; checked_at: number }
  | { kind: "available"; version: string; notes: string }
  | { kind: "installing"; percent: number | null }
  | { kind: "error"; message: string; during: "check" | "install" };

export const getUpdateStatus = () => invoke<UpdateStatus>("get_update_status");
export const checkForUpdate = () => invoke<UpdateStatus>("check_for_update");
export const installUpdate = () => invoke<void>("install_update");

/** Live update status: the current value, then every `update-status` event. */
export function useUpdateStatus(): UpdateStatus {
  const [status, setStatus] = useState<UpdateStatus>({ kind: "idle" });
  useEffect(() => {
    getUpdateStatus().then(setStatus).catch(() => {});
    const un = listen<UpdateStatus>("update-status", (e) => setStatus(e.payload));
    return () => {
      un.then((f) => f());
    };
  }, []);
  return status;
}

export function useAppVersion(): string {
  const [version, setVersion] = useState("");
  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
  }, []);
  return version;
}
