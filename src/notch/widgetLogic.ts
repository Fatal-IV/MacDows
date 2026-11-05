// Rules and formatting for the status widgets. Pure functions; the components only draw.
import type { Widgets } from "../bridge";

export type WidgetId = "weather" | "battery" | "cpu" | "ram" | "disk" | "net";
export const WIDGET_IDS: WidgetId[] = ["weather", "battery", "cpu", "ram", "disk", "net"];
export const MAX_PER_SIDE = 3;
export const WEATHER_CACHE_MS = 30 * 60_000;

export interface Placed {
  left: WidgetId[];
  right: WidgetId[];
}

const isId = (id: string): id is WidgetId => (WIDGET_IDS as string[]).includes(id);

/** The placed widgets that can be shown now: no battery on a desktop, no weather without data. */
export function visibleWidgets(w: Widgets, ctx: { hasBattery: boolean; weatherReady: boolean }): Placed {
  const keep = (ids: string[]) => ids.filter(isId).filter((id) => (id !== "battery" || ctx.hasBattery) && (id !== "weather" || ctx.weatherReady));
  return { left: keep(w.left), right: keep(w.right) };
}

export function widgetsWidth(v: Placed): number {
  return [...v.left, ...v.right].reduce((sum, id) => sum + (id === "net" ? 84 : 54), 0);
}

export const needsStats = (v: Placed): boolean => [...v.left, ...v.right].some((id) => id === "cpu" || id === "ram" || id === "disk" || id === "net");

/** "↓ 1,2 MB/s"; below 1 KB/s just "0". */
export function formatRate(bytesPerSecond: number): string {
  if (bytesPerSecond < 1024) return "0";
  if (bytesPerSecond < 1024 * 1024) return `↓ ${Math.round(bytesPerSecond / 1024)} KB/s`;
  return `↓ ${(bytesPerSecond / (1024 * 1024)).toFixed(1).replace(".", ",")} MB/s`;
}

export function formatTemp(celsius: number, unit: "celsius" | "fahrenheit"): string {
  const v = unit === "fahrenheit" ? (celsius * 9) / 5 + 32 : celsius;
  return `${Math.round(v) + 0}°`;
}

export type WeatherIcon = "clear" | "partly" | "cloudy" | "fog" | "rain" | "snow" | "storm";

/** Open-Meteo WMO weather code to one of our icons. */
export function weatherIcon(code: number): WeatherIcon {
  if (code === 0 || code === 1) return "clear";
  if (code === 2) return "partly";
  if (code === 3) return "cloudy";
  if (code === 45 || code === 48) return "fog";
  if ((code >= 51 && code <= 67) || (code >= 80 && code <= 82)) return "rain";
  if ((code >= 71 && code <= 77) || code === 85 || code === 86) return "snow";
  if (code >= 95 && code <= 99) return "storm";
  return "cloudy";
}

export const isLowBattery = (percent: number, threshold: number, charging: boolean): boolean => !charging && percent < threshold;

export const trayWidgets = (w: Widgets): WidgetId[] => WIDGET_IDS.filter((id) => !w.left.includes(id) && !w.right.includes(id));

/** Moves a widget to a side (at `index`) or back to the tray. Null when the side already holds three. */
export function moveWidget(w: Widgets, id: WidgetId, to: { side: "left" | "right" | "tray"; index: number }): Widgets | null {
  const next: Widgets = { left: w.left.filter((x) => x !== id), right: w.right.filter((x) => x !== id) };
  if (to.side === "tray") return next;
  const side = next[to.side];
  if (side.length >= MAX_PER_SIDE) return null;
  side.splice(Math.max(0, Math.min(to.index, side.length)), 0, id);
  return next;
}

export const weatherCacheFresh = (savedAtMs: number, nowMs: number): boolean => nowMs >= savedAtMs && nowMs - savedAtMs < WEATHER_CACHE_MS;
