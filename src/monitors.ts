import type { MonitorCfg, MonitorView } from "./bridge";

/** One monitor as the Settings section shows it: a plugged-in one (from Windows) or a saved one that is not connected right now. */
export interface MonitorCard extends MonitorView {
  connected: boolean;
}

/** The plugged-in monitors in the order they sit on the desk (left to right), then the saved ones that are not connected. */
export function monitorCards(live: MonitorView[], saved: MonitorCfg[]): MonitorCard[] {
  const cards: MonitorCard[] = [...live].sort((a, b) => a.x - b.x || a.y - b.y).map((m) => ({ ...m, connected: true }));
  for (const s of saved) {
    if (live.some((m) => m.key === s.key)) continue;
    cards.push({ key: s.key, name: s.name, width: 0, height: 0, x: 0, y: 0, primary: false, island: s.island, dock: s.dock, connected: false });
  }
  return cards;
}

/** The saved list with one flag of one monitor changed (the entry is created when there is none; the other flag keeps its current value). */
export function setFlag(saved: MonitorCfg[], card: MonitorCard, flag: "island" | "dock", on: boolean): MonitorCfg[] {
  const base: MonitorCfg = saved.find((s) => s.key === card.key) ?? { key: card.key, name: card.name, island: card.island, dock: card.dock };
  const next: MonitorCfg = { ...base, [flag]: on };
  return saved.some((s) => s.key === card.key) ? saved.map((s) => (s.key === card.key ? next : s)) : [...saved, next];
}

/** Only with two monitors or more there is anything to choose. */
export function showSection(cards: MonitorCard[]): boolean {
  return cards.length >= 2;
}
