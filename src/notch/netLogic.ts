import type { VpnState, WifiNetwork } from "../bridge";

/** 0-100 signal quality to 0-4 bars (same thresholds as the Rust side). */
export function signalBars(signal: number): number {
  if (signal <= 0) return 0;
  if (signal <= 25) return 1;
  if (signal <= 50) return 2;
  if (signal <= 75) return 3;
  return 4;
}

export function securityLabel(n: WifiNetwork, t: (key: string) => string): string {
  if (!n.secured) return t("net.open");
  return n.enterprise ? t("net.enterprise") : t("net.secured");
}

const ERROR_KEYS = ["not_found", "failed", "timeout", "needs_password", "enterprise", "wrong_password"];

/** The locale key for a backend error key; unknown ones read as a plain failure. */
export function errorText(key: string, t: (key: string) => string): string {
  return t(`net.err.${ERROR_KEYS.includes(key) ? key : "failed"}`);
}

export type WifiAuth = "open" | "wpa2" | "wpa3";

/** Whether the join form can be sent: an SSID, and a password of the right length for the security type. */
export function canSubmit(ssid: string, auth: WifiAuth, password: string): boolean {
  if (ssid.trim().length === 0) return false;
  const n = [...password].length;
  if (auth === "wpa2") return n >= 8 && n <= 63;
  if (auth === "wpa3") return n >= 1;
  return true;
}

/** The VPN card's state line: connecting, on, on through an app, or off. */
export function vpnLabel(state: VpnState | null, t: (key: string) => string): string {
  if (!state) return t("vpn.off");
  if (state.entries.some((e) => e.state === "connecting")) return t("vpn.connecting");
  if (state.entries.some((e) => e.state === "connected")) return t("vpn.on");
  return state.app_vpn_active ? t("vpn.onApp") : t("vpn.off");
}
