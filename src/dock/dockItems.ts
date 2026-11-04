import type { AppWindow, PinnedApp } from "../bridge";

export interface DockEntry {
  /** Stable React key. */
  key: string;
  title: string;
  icon: string | null;
  /** Set for a pinned app; clicking it starts the app when it has no window. */
  pinned: PinnedApp | null;
  /** Windows of this app, in dock order. */
  windows: AppWindow[];
  /** A pinned app that has a window shows a dot under its icon. */
  running: boolean;
  /** The program and Store app id, for the right-click menu. */
  exe: string;
  aumid: string;
}

const same = (a: string, b: string) => a !== "" && b !== "" && a.replace(/\//g, "\\").toLowerCase() === b.replace(/\//g, "\\").toLowerCase();
/** A window belongs to a pin by its program, or by its app id (Store apps). */
const belongs = (w: AppWindow, p: PinnedApp) => same(w.exe, p.exe) || same(w.aumid, p.aumid);

/** Pinned apps first (a running one keeps its place and absorbs its windows), then the windows that belong to no pinned app. */
export function mergeDock(pinned: PinnedApp[], windows: AppWindow[]): DockEntry[] {
  const claimed = new Set<number>();
  const entries: DockEntry[] = pinned.map((p) => {
    const own = windows.filter((w) => belongs(w, p));
    own.forEach((w) => claimed.add(w.id));
    return { key: `pin:${p.id}`, title: p.name, icon: p.icon_png_base64, pinned: p, windows: own, running: own.length > 0, exe: p.exe, aumid: p.aumid };
  });
  for (const w of windows) {
    if (!claimed.has(w.id)) entries.push({ key: `win:${w.id}`, title: w.title, icon: w.icon_png_base64, pinned: null, windows: [w], running: false, exe: w.exe, aumid: w.aumid });
  }
  return entries;
}
