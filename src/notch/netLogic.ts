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
