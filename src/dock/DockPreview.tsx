import { useEffect, useState } from "react";
import { closePreview, onPreviewUpdated, previewActivate, previewCloseWindow, previewHover, previewItems, type PreviewItem } from "../bridge";
import { initTheme } from "../appearance/theme";
import "./Preview.css";

const COLUMNS = 5;

/** The window previews that open above a running app in the dock: one card per window. */
export default function DockPreview() {
  const [items, setItems] = useState<PreviewItem[]>([]);

  useEffect(() => initTheme(), []);

  useEffect(() => {
    const load = () => previewItems().then(setItems).catch(() => setItems([]));
    load();
    const un = onPreviewUpdated(load);
    return () => {
      un.then((f) => f());
    };
  }, []);

  // The browser's own right-click menu makes no sense here.
  useEffect(() => {
    const block = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", block);
    return () => document.removeEventListener("contextmenu", block);
  }, []);

  return (
    <div
      className="preview-panel"
      style={{ gridTemplateColumns: `repeat(${Math.min(COLUMNS, items.length || 1)}, 200px)` }}
      onPointerEnter={() => previewHover("panel", true).catch(() => {})}
      onPointerLeave={() => previewHover("panel", false).catch(() => {})}
    >
      {items.map((item) => (
        <div key={item.hwnd} className="preview-card" role="button" tabIndex={-1} title={item.title} onClick={() => previewActivate(item.hwnd).catch(() => closePreview())}>
          <div className="preview-head">
            {item.icon && <img className="preview-icon" src={`data:image/png;base64,${item.icon}`} alt="" draggable={false} />}
            <span className="preview-title">{item.title}</span>
            <button
              className="preview-close"
              aria-label="×"
              onClick={(e) => {
                e.stopPropagation();
                previewCloseWindow(item.hwnd).catch(() => {});
              }}
            >
              ×
            </button>
          </div>
          <div className="preview-body">
            {item.thumbnail ? (
              <img src={`data:image/png;base64,${item.thumbnail}`} alt="" draggable={false} />
            ) : item.icon ? (
              <img className="preview-fallback" src={`data:image/png;base64,${item.icon}`} alt="" draggable={false} />
            ) : null}
          </div>
        </div>
      ))}
    </div>
  );
}
