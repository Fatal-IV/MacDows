import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { focusWindow, listWindows, onWindowsChanged, type AppWindow } from "../bridge";
import { useT } from "../i18n";
import { shellAction, useSystemStatus, type ShellAction } from "../system";
import "./Dock.css";

export default function Dock() {
  const t = useT();
  const system = useSystemStatus();
  const [windows, setWindows] = useState<AppWindow[]>([]);

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
      <motion.div className="dock" layout>
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
            <span className="dock-dot" />
          </motion.button>
        ))}
        {system?.taskbar_hidden && (
          <>
            <span className="dock-sep" />
            <SystemButton action="notifications" label={t("system.notifications")} icon={BELL} />
            <SystemButton action="quick_settings" label={t("system.quickSettings")} icon={SLIDERS} />
            {system.hidden_icons_available && <SystemButton action="hidden_icons" label={t("system.hiddenIcons")} icon={CHEVRON} />}
          </>
        )}
      </motion.div>
    </div>
  );
}

const BELL = "M12 3a5 5 0 0 0-5 5v3l-1.5 3h13L17 11V8a5 5 0 0 0-5-5Zm-2 14a2 2 0 0 0 4 0";
const SLIDERS = "M5 7h8m4 0h2M5 17h2m4 0h8M13 4v6m-4 4v6";
const CHEVRON = "M7 14l5-5 5 5";

function SystemButton({ action, label, icon }: { action: ShellAction; label: string; icon: string }) {
  return (
    <motion.button
      className="dock-system"
      title={label}
      aria-label={label}
      layout
      whileHover={{ scale: 1.15 }}
      whileTap={{ scale: 0.95 }}
      onClick={() => shellAction(action).catch(() => {})}
    >
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">
        <path d={icon} />
      </svg>
    </motion.button>
  );
}
