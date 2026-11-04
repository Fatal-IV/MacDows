import { useEffect, useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { focusWindow, launchPinned, listPinned, pinnedMenu, listWindows, onPinnedChanged, onWindowsChanged, type AppWindow, type PinnedApp } from "../bridge";
import { mergeDock, type DockEntry } from "./dockItems";
import { useSettings, useT } from "../i18n";
import { initTheme } from "../appearance/theme";
import { shellAction, useSystemStatus, type ShellAction } from "../system";
import "./Dock.css";

export default function Dock() {
  const t = useT();
  const settings = useSettings();
  const system = useSystemStatus();
  const [windows, setWindows] = useState<AppWindow[]>([]);
  const [pinned, setPinned] = useState<PinnedApp[]>([]);
  const showPinned = settings?.show_pinned_apps ?? true;
  // The dock reloads the pins when the list of ones hidden in the dock changes.
  const hiddenKey = (settings?.hidden_pinned ?? []).join("|");

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

  return (
    <div className="dock-stage">
      <motion.div className="dock" layout>
        {system?.taskbar_hidden && (
          <>
            <SystemButton action="start" label={t("system.start")} icon={START} />
            <SystemButton action="search" label={t("system.search")} icon={SEARCH} />
            <span className="dock-sep" />
          </>
        )}
        {entries.length === 0 && <div className="dock-empty">{t("dock.noApps")}</div>}
        {entries.map((e) => (
          <motion.button
            key={e.key}
            className={`dock-item${e.running ? " running" : ""}`}
            title={e.title}
            layout
            whileHover={{ scale: 1.25, y: -6 }}
            whileTap={{ scale: 1.1 }}
            transition={{ type: "spring", stiffness: 400, damping: 22 }}
            onClick={() => open(e)}
            onContextMenu={(ev) => {
              if (!e.pinned) return;
              ev.preventDefault();
              pinnedMenu(e.pinned.id).catch(() => {});
            }}
          >
            {e.icon ? <img src={`data:image/png;base64,${e.icon}`} alt="" draggable={false} /> : <span className="dock-fallback">{e.title.slice(0, 1).toUpperCase()}</span>}
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
    </div>
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
