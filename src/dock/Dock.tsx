import { useEffect, useRef, useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { focusWindow, listWindows, onWindowsChanged, type AppWindow } from "../bridge";
import { useT } from "../i18n";
import { initTheme } from "../appearance/theme";
import { shellAction, useSystemStatus, type ShellAction } from "../system";
import "./Dock.css";

export default function Dock() {
  const t = useT();
  const system = useSystemStatus();
  const [windows, setWindows] = useState<AppWindow[]>([]);

  useEffect(() => initTheme({ backdrop: true }), []);

  // The menus opened from the keyboard (Win, Win+S) need to know where the dock's top edge is.
  const dockRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = dockRef.current;
    if (!el) return;
    const report = () => invoke("report_dock_top", { top: el.getBoundingClientRect().top }).catch(() => {});
    report();
    const observer = new ResizeObserver(report);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  useEffect(() => {
    const refresh = () => listWindows().then(setWindows).catch(() => setWindows([]));
    refresh();
    const un = onWindowsChanged(refresh);
    return () => {
      un.then((f) => f());
    };
  }, []);

  return (
    <div className="dock-stage">
      <motion.div ref={dockRef} className="dock" layout>
        {system?.taskbar_hidden && (
          <>
            <SystemButton action="start" label={t("system.start")} icon={START} onClick={openStartMenu} />
            <SystemButton action="search" label={t("system.search")} icon={SEARCH} onClick={openSearchMenu} />
            <span className="dock-sep" />
          </>
        )}
        {windows.length === 0 && <div className="dock-empty">{t("dock.noApps")}</div>}
        {windows.map((w) => (
          <motion.button
            key={w.id}
            className="dock-item"
            title={w.title}
            layout
            whileHover={{ scale: 1.25, y: -6 }}
            whileTap={{ scale: 1.1 }}
            transition={{ type: "spring", stiffness: 400, damping: 22 }}
            onClick={() => focusWindow(w.id)}
          >
            {w.icon_png_base64 ? (
              <img src={`data:image/png;base64,${w.icon_png_base64}`} alt="" draggable={false} />
            ) : (
              <span className="dock-fallback">{w.title.slice(0, 1).toUpperCase()}</span>
            )}
          </motion.button>
        ))}
        {system?.taskbar_hidden && (
          <>
            <span className="dock-sep" />
            <SystemButton action="notifications" label={t("system.notifications")} icon={BELL} />
            <SystemButton action="quick_settings" label={t("system.quickSettings")} icon={SLIDERS} />
            <SystemButton action="hidden_icons" label={t("system.hiddenIcons")} icon={CHEVRON} onClick={openTrayPanel} />
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
function dockTop(button: HTMLElement) {
  return button.closest(".dock")?.getBoundingClientRect().top ?? button.getBoundingClientRect().top;
}

/** Opens the MacDows Start menu above the dock. */
function openStartMenu(button: HTMLElement) {
  invoke("toggle_start_menu", { dockTop: dockTop(button) }).catch(() => {});
}

/** Opens the MacDows search menu above the dock. */
function openSearchMenu(button: HTMLElement) {
  invoke("toggle_search_menu", { dockTop: dockTop(button) }).catch(() => {});
}

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
