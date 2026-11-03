import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { initTheme } from "../appearance/theme";
import "./Tray.css";

interface TrayItem {
  index: number;
  name: string;
  icon_png_base64: string | null;
}

const COLUMNS = 5;
// A blur that fires while the window is still being focused would close it at once.
const BLUR_GRACE_MS = 400;

/** The hidden notification-area icons, drawn with the app theme above the dock. */
export default function TrayPanel() {
  const [items, setItems] = useState<TrayItem[]>([]);

  useEffect(() => initTheme({ backdrop: true }), []);
  useEffect(() => {
    invoke<TrayItem[]>("get_tray_items").then(setItems).catch(() => setItems([]));
  }, []);

  useEffect(() => {
    const opened = Date.now();
    const close = () => invoke("close_tray_panel").catch(() => {});
    const un = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused && Date.now() - opened > BLUR_GRACE_MS) close();
    });
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => {
      un.then((f) => f());
      window.removeEventListener("keydown", onKey);
    };
  }, []);

  return (
    <div className="tray-panel" style={{ gridTemplateColumns: `repeat(${Math.min(COLUMNS, items.length || 1)}, 1fr)` }}>
      {items.map((item) => (
        <button
          key={item.index}
          className="tray-item"
          title={item.name}
          aria-label={item.name}
          onClick={() => invoke("activate_tray_item", { index: item.index, name: item.name }).catch(() => {})}
        >
          {item.icon_png_base64 ? (
            <img src={`data:image/png;base64,${item.icon_png_base64}`} alt="" draggable={false} />
          ) : (
            <span className="tray-fallback">{item.name.slice(0, 1).toUpperCase()}</span>
          )}
        </button>
      ))}
    </div>
  );
}
