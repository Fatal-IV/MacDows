import { useEffect, useRef, useState, type ReactNode } from "react";
import { motion, Reorder } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { isFirstRun, focusWindow, dockMenu, getCustomIcons, previewHover, showPreview, launchNewApp, launchPinned, listDrives, listPinned, onDockShortcut, onDrivesChanged, openDrive, setDockPinnedCount, setPinnedOrder, listWindows, onPinnedChanged, onWindowsChanged, type AppWindow, type Drive, type PinnedApp } from "../bridge";
import { mergeDock, type DockEntry } from "./dockItems";
import { droppedOutside, sameOrder } from "./dockDrag";
import { useSettings, useT } from "../i18n";
import { initTheme } from "../appearance/theme";
import { dockBox, type Box } from "../hitBox";
import { useHitRegion } from "../hitRegion";
import { shellAction, useSystemStatus, type ShellAction } from "../system";
import "./Dock.css";

export default function Dock() {
  const t = useT();
  const settings = useSettings();
  const system = useSystemStatus();
  // The dock's entrance: held back until the backend says whether this is the very first start, then it rises with a bounce.
  const [intro, setIntro] = useState<"pending" | "first" | "normal">("pending");
  useEffect(() => {
    isFirstRun().then((first) => setIntro(first ? "first" : "normal")).catch(() => setIntro("normal"));
  }, []);
  // The dock's own shape in its window (layout position, so the entrance transform does not matter): only that takes clicks.
  const [dockShape, setDockShape] = useState<Box | null>(null);
  const [windows, setWindows] = useState<AppWindow[]>([]);
  const [pinned, setPinned] = useState<PinnedApp[]>([]);
  const showPinned = settings?.show_pinned_apps ?? true;
  // The dock reloads the pins when the list of ones hidden in the dock changes.
  const hiddenKey = (settings?.hidden_pinned ?? []).join("|");
  const [drives, setDrives] = useState<Drive[]>([]);
  const showDrives = settings?.dock?.show_drives ?? true;
  // Icons the user picked; reloaded when the list of them changes.
  const [customIcons, setCustomIcons] = useState<Record<string, string>>({});
  const iconsKey = JSON.stringify(settings?.dock?.custom_icons ?? {});

  useEffect(() => initTheme({ backdrop: true }), []);

  // The browser's own right-click menu ("Save image as", "Inspect"...) makes no sense on a dock.
  useEffect(() => {
    const block = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", block);
    return () => document.removeEventListener("contextmenu", block);
  }, []);

  useEffect(() => {
    const refresh = () => listWindows().then(setWindows).catch(() => setWindows([]));
    refresh();
    const un = onWindowsChanged(refresh);
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    getCustomIcons().then(setCustomIcons).catch(() => setCustomIcons({}));
  }, [iconsKey]);

  // Local and removable drives; removable ones come and go.
  useEffect(() => {
    if (!showDrives) {
      setDrives([]);
      return;
    }
    const refresh = () => listDrives().then(setDrives).catch(() => setDrives([]));
    refresh();
    const un = onDrivesChanged(refresh);
    return () => {
      un.then((f) => f());
    };
  }, [showDrives]);

  // The apps pinned to the Windows taskbar; reloaded when Windows changes them.
  useEffect(() => {
    if (!showPinned) {
      setPinned([]);
      return;
    }
    const refresh = () => listPinned().then(setPinned).catch(() => setPinned([]));
    refresh();
    const un = onPinnedChanged(refresh);
    return () => {
      un.then((f) => f());
    };
  }, [showPinned, hiddenKey]);

  const entries = mergeDock(pinned, windows);
  const open = (e: DockEntry) => {
    if (e.windows.length > 0) focusWindow(e.windows[0].id);
    else if (e.pinned) launchPinned(e.pinned.id).catch(() => {});
  };

  // Dragging the pinned apps: `order` holds the ids while one is being moved (null = the order Windows and the saved order give).
  const dockRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = dockRef.current;
    if (!el) return;
    const measure = () => setDockShape({ x: el.offsetLeft, y: el.offsetTop, w: el.offsetWidth, h: el.offsetHeight });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    window.addEventListener("resize", measure);
    return () => {
      observer.disconnect();
      window.removeEventListener("resize", measure);
    };
  }, []);
  // Side margins hold the flares that blend the dock into the screen edge; room above holds the hover pop-up of the icons.
  useHitRegion("dock", dockShape ? dockBox(dockShape, window.innerHeight, 20, 44) : null, 0);
  const [order, setOrder] = useState<string[] | null>(null);
  const orderRef = useRef<string[] | null>(null);
  const dragFrom = useRef<string[] | null>(null);
  const cancelled = useRef(false);
  const lastDragEnd = useRef(0);
  const updateOrder = (ids: string[] | null) => {
    orderRef.current = ids;
    setOrder(ids);
  };
  // A fresh list from Rust (after saving, or when Windows changes the pins) replaces the one being dragged.
  useEffect(() => updateOrder(null), [pinned]);
  // Esc during a drag puts everything back.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && dragFrom.current) {
        cancelled.current = true;
        updateOrder(dragFrom.current);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const pinnedEntries = entries.filter((e) => e.pinned);
  const otherEntries = entries.filter((e) => !e.pinned);
  const baseIds = pinnedEntries.map((e) => e.pinned!.id);
  const ids = order && sameOrder([...order].sort(), [...baseIds].sort()) ? order : baseIds;
  const byId = new Map(pinnedEntries.map((e) => [e.pinned!.id, e]));

  // Win+1..9: the nth pinned app (in the order shown) comes forward or starts; Shift starts a new window.
  const shown = ids.map((id) => byId.get(id)).filter((e): e is DockEntry => !!e);
  const shortcutState = useRef({ shown, cycle: { index: -1, at: 0 } });
  shortcutState.current.shown = shown;
  const inCharge = !!system?.taskbar_hidden;
  useEffect(() => {
    setDockPinnedCount(inCharge ? shown.length : 0).catch(() => {});
  }, [shown.length, inCharge]);
  useEffect(() => {
    const un = onDockShortcut(({ index, new_instance }) => {
      const st = shortcutState.current;
      const entry = st.shown[index];
      if (!entry) return;
      if (new_instance) {
        launchNewApp(entry.pinned?.id ?? "", entry.exe, entry.aumid).catch(() => {});
      } else if (entry.windows.length > 0) {
        // Pressing the same number again walks through the app's windows.
        st.cycle = { index, at: st.cycle.index === index ? (st.cycle.at + 1) % entry.windows.length : 0 };
        focusWindow(entry.windows[st.cycle.at].id);
      } else if (entry.pinned) {
        launchPinned(entry.pinned.id).catch(() => {});
      }
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  // Window previews: hover a running app for a moment and a panel with its windows opens above the icon.
  const previewsOn = settings?.dock?.previews ?? true;
  const hoverTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const dragging = useRef(false);
  const hoverStart = (e: DockEntry, button: HTMLElement) => {
    if (!previewsOn || e.windows.length === 0) return;
    previewHover("dock", true).catch(() => {});
    clearTimeout(hoverTimer.current);
    hoverTimer.current = setTimeout(() => {
      if (dragging.current) return;
      const r = button.getBoundingClientRect();
      const dockTop = dockRef.current?.getBoundingClientRect().top ?? r.top;
      const icon = customIcons[(e.pinned?.id ?? e.exe).toLowerCase()] ?? e.icon;
      showPreview(r.left + r.width / 2, dockTop, icon, e.windows.map((w) => w.id)).catch(() => {});
    }, 300);
  };
  const hoverEnd = () => {
    clearTimeout(hoverTimer.current);
    previewHover("dock", false).catch(() => {});
  };
  const badge = (e: DockEntry) => (e.windows.length >= 2 ? <span className="dock-badge">{e.windows.length}</span> : null);

  const onDragStart = () => {
    dragging.current = true;
    hoverEnd();
    dragFrom.current = ids;
    cancelled.current = false;
  };
  const onDragEnd = (_: unknown, info: { point: { y: number } }) => {
    dragging.current = false;
    const start = dragFrom.current;
    dragFrom.current = null;
    lastDragEnd.current = Date.now();
    const rect = dockRef.current?.getBoundingClientRect();
    const away = rect ? droppedOutside(info.point.y, rect.top, rect.bottom) : false;
    const next = orderRef.current;
    if (cancelled.current || away || !start || !next) {
      updateOrder(null);
      return;
    }
    if (!sameOrder(start, next)) setPinnedOrder(next).catch(() => updateOrder(null));
  };
  // A click right after a drag is the end of the drag, not a request to open the app.
  const openUnlessDragged = (e: DockEntry) => {
    if (Date.now() - lastDragEnd.current > 250) open(e);
  };
  const menuFor = (e: DockEntry) => (ev: React.MouseEvent) => {
    ev.preventDefault();
    ev.stopPropagation();
    dockMenu({ kind: e.pinned ? "pinned" : "running", id: e.pinned?.id ?? "", exe: e.exe, aumid: e.aumid, name: e.title, hwnds: e.windows.map((w) => w.id) }).catch(() => {});
  };
  const iconOf = (e: DockEntry) => {
    const icon = customIcons[(e.pinned?.id ?? e.exe).toLowerCase()] ?? e.icon;
    return icon ? <img src={`data:image/png;base64,${icon}`} alt="" draggable={false} /> : <span className="dock-fallback">{e.title.slice(0, 1).toUpperCase()}</span>;
  };
  const itemTransition = { type: "spring", stiffness: 400, damping: 22 } as const;

  return (
    <motion.div
      className="dock-stage"
      initial={{ y: 90, opacity: 0 }}
      animate={intro === "pending" ? { y: 90, opacity: 0 } : { y: 0, opacity: 1 }}
      transition={intro === "first" ? { type: "spring", stiffness: 120, damping: 11, delay: 0.9 } : { duration: 0 }}
      onContextMenu={(ev) => {
        // Right-click on the dock's empty space (not on an icon).
        ev.preventDefault();
        dockMenu({ kind: "empty", id: "", exe: "", aumid: "", name: "", hwnds: [] }).catch(() => {});
      }}
    >
      <motion.div className="dock" layout ref={dockRef}>
        {system?.taskbar_hidden && (
          <>
            <SystemButton action="start" label={t("system.start")} icon={START} />
            <SystemButton action="search" label={t("system.search")} icon={SEARCH} />
            <span className="dock-sep" />
          </>
        )}
        {entries.length === 0 && <div className="dock-empty">{t("dock.noApps")}</div>}
        <Reorder.Group axis="x" values={ids} onReorder={updateOrder} as="div" className="dock-pins">
          {ids.map((id) => {
            const e = byId.get(id);
            if (!e) return null;
            return (
              <Reorder.Item
                key={id}
                value={id}
                as="button"
                className={`dock-item${e.running ? " running" : ""}`}
                title={previewsOn && e.windows.length > 0 ? undefined : e.title}
                whileHover={{ scale: 1.25, y: -6 }}
                whileTap={{ scale: 1.1 }}
                whileDrag={{ scale: 1.15, zIndex: 5 }}
                transition={itemTransition}
                onDragStart={onDragStart}
                onDragEnd={onDragEnd}
                onClick={() => openUnlessDragged(e)}
                onContextMenu={menuFor(e)}
                onPointerEnter={(ev) => hoverStart(e, ev.currentTarget as HTMLElement)}
                onPointerLeave={hoverEnd}
              >
                {iconOf(e)}
                {badge(e)}
              </Reorder.Item>
            );
          })}
        </Reorder.Group>
        {drives.length > 0 && (
          <>
            <span className="dock-sep" />
            {drives.map((dr) => (
              <motion.button
                key={dr.letter}
                className="dock-item dock-drive"
                title={`${dr.label || t(dr.kind === "removable" ? "dock.drive.removable" : "dock.drive.local")} (${dr.letter}:)`}
                layout
                whileHover={{ scale: 1.25, y: -6 }}
                whileTap={{ scale: 1.1 }}
                transition={itemTransition}
                onClick={() => openDrive(dr.letter).catch(() => {})}
              >
                <svg viewBox="0 0 32 32" width="30" height="30" aria-hidden>
                  <rect x="3" y="9" width="26" height="14" rx="3" fill="none" stroke="currentColor" strokeWidth="1.8" />
                  <circle cx="24" cy="16" r="1.4" fill={dr.kind === "removable" ? "currentColor" : "none"} stroke="currentColor" strokeWidth="1.4" />
                  <path d="M7 16h8" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
                </svg>
                <span className="dock-drive-letter">{dr.letter}</span>
              </motion.button>
            ))}
            <span className="dock-sep" />
          </>
        )}
        {otherEntries.map((e) => (
          <motion.button key={e.key} className="dock-item running" title={previewsOn && e.windows.length > 0 ? undefined : e.title} layout whileHover={{ scale: 1.25, y: -6 }} whileTap={{ scale: 1.1 }} transition={itemTransition} onClick={() => open(e)} onContextMenu={menuFor(e)} onPointerEnter={(ev) => hoverStart(e, ev.currentTarget as HTMLElement)} onPointerLeave={hoverEnd}>
            {iconOf(e)}
            {badge(e)}
          </motion.button>
        ))}
        {system?.taskbar_hidden && (
          <>
            <span className="dock-sep" />
            <SystemButton action="notifications" label={t("system.notifications")} icon={BELL} />
            <SystemButton action="quick_settings" label={t("system.quickSettings")} icon={SLIDERS} />
            <SystemButton action="hidden_icons" label={t("system.hiddenIcons")} icon={CHEVRON} onClick={openTrayPanel} />
            {settings?.show_desktop_button && (
              <>
                <span className="dock-sep" />
                <SystemButton action="show_desktop" label={t("system.showDesktop")} icon={DESKTOP} />
              </>
            )}
          </>
        )}
      </motion.div>
    </motion.div>
  );
}

const stroke = (d: string) => <path d={d} fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />;
const BELL = stroke("M12 3a5 5 0 0 0-5 5v3l-1.5 3h13L17 11V8a5 5 0 0 0-5-5Zm-2 14a2 2 0 0 0 4 0");
const SLIDERS = stroke("M5 7h8m4 0h2M5 17h2m4 0h8M13 4v6m-4 4v6");
const CHEVRON = stroke("M7 14l5-5 5 5");
const DESKTOP = stroke("M4 5h16v10H4zM9 19h6M12 15v4");
const SEARCH = (
  <>
    <circle cx="10.5" cy="10.5" r="5.5" fill="none" stroke="currentColor" strokeWidth="1.8" />
    {stroke("M15 15l5 5")}
  </>
);
// The Windows logo: four panes.
const START = (
  <g fill="currentColor">
    <rect x="4" y="4" width="7.2" height="7.2" rx="1" />
    <rect x="12.8" y="4" width="7.2" height="7.2" rx="1" />
    <rect x="4" y="12.8" width="7.2" height="7.2" rx="1" />
    <rect x="12.8" y="12.8" width="7.2" height="7.2" rx="1" />
  </g>
);

/** Opens MacDows's own hidden-icons panel above the dock, centered on the button. */
function openTrayPanel(button: HTMLElement) {
  const r = button.getBoundingClientRect();
  const dock = button.closest(".dock")?.getBoundingClientRect();
  invoke("toggle_tray_panel", { centerX: r.left + r.width / 2, dockTop: dock?.top ?? r.top }).catch(() => {});
}

function SystemButton({ action, label, icon, onClick }: { action: ShellAction; label: string; icon: ReactNode; onClick?: (button: HTMLElement) => void }) {
  return (
    <motion.button
      className="dock-system"
      title={label}
      aria-label={label}
      layout
      whileHover={{ scale: 1.15 }}
      whileTap={{ scale: 0.95 }}
      onClick={(e) => (onClick ? onClick(e.currentTarget) : shellAction(action).catch(() => {}))}
    >
      <svg viewBox="0 0 24 24" width="18" height="18">
        {icon}
      </svg>
    </motion.button>
  );
}
