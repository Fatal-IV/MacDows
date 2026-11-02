import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { focusWindow, listWindows, onWindowsChanged, type AppWindow } from "../bridge";
import "./Dock.css";

export default function Dock() {
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
        {windows.length === 0 && <div className="dock-empty">No open apps</div>}
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
      </motion.div>
    </div>
  );
}
