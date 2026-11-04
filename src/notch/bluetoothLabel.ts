import type { BtDevice } from "../bridge";

/** What the Bluetooth tile says when it is on: the connected device's name ("Name +1" for more), else `fallback`. */
export function connectedLabel(devices: Pick<BtDevice, "name" | "connected">[] | null, fallback: string): string {
  const names = (devices ?? []).filter((d) => d.connected && d.name.trim() !== "").map((d) => d.name.trim());
  if (names.length === 0) return fallback;
  return names.length === 1 ? names[0] : `${names[0]} +${names.length - 1}`;
}
