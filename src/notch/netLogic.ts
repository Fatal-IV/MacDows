import type { WifiNetwork } from "../bridge";

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
