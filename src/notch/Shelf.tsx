// The shelf: files dropped on the notch, kept by path. Click opens, a drag takes the file to another app.
import { useRef } from "react";
import { startDrag } from "@crabnebula/tauri-plugin-drag";
import { openShelfItem, revealShelfItem, type ShelfItem } from "../bridge";
import { useT } from "../i18n";

export function DropZone() {
  const t = useT();
  return (
    <div className="dropzone">
      <svg width="30" height="30" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
        <path d="M12 4v11M7.5 10.5 12 15l4.5-4.5M5 19h14" />
      </svg>
      <span>{t("shelf.drop")}</span>
    </div>
  );
}

function Tile({ item, onRemove }: { item: ShelfItem; onRemove: (path: string) => void }) {
  const t = useT();
  const press = useRef<{ x: number; y: number; dragged: boolean } | null>(null);

  const onPointerDown = (e: React.PointerEvent) => {
    if (e.button !== 0 || item.missing) return;
    press.current = { x: e.clientX, y: e.clientY, dragged: false };
  };
  const onPointerMove = (e: React.PointerEvent) => {
    const p = press.current;
    if (!p || p.dragged || !item.icon_path) return;
    if (Math.hypot(e.clientX - p.x, e.clientY - p.y) < 6) return;
    // From here on the file is dragged out by Windows itself, to whatever app it is dropped on.
    p.dragged = true;
    startDrag({ item: [item.path], icon: item.icon_path }).catch(() => {});
  };
  const onClick = () => {
    const dragged = press.current?.dragged;
    press.current = null;
    if (!dragged && !item.missing) openShelfItem(item.path).catch(() => {});
  };

  return (
    <div
      className={`shelf-tile${item.missing ? " missing" : ""}`}
      title={item.missing ? t("shelf.missing") : item.path}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={() => (press.current = null)}
      onClick={onClick}
      onContextMenu={(e) => {
        e.preventDefault();
        revealShelfItem(item.path).catch(() => {});
      }}
    >
      <span className="shelf-icon">{item.icon_png_base64 ? <img src={`data:image/png;base64,${item.icon_png_base64}`} alt="" draggable={false} /> : <span>{item.is_dir ? "▣" : "▤"}</span>}</span>
      <span className="shelf-name">{item.name}</span>
      <button
        className="shelf-x"
        aria-label={t("shelf.remove")}
        title={t("shelf.remove")}
        onPointerDown={(e) => e.stopPropagation()}
        onClick={(e) => {
          e.stopPropagation();
          onRemove(item.path);
        }}
      >
        ✕
      </button>
    </div>
  );
}

export function ShelfScreen({ items, onRemove, onClear }: { items: ShelfItem[]; onRemove: (path: string) => void; onClear: () => void }) {
  const t = useT();
  return (
    <div className="shelf">
      <div className="shelf-head">
        <span className="shelf-title">{t("shelf.title")}</span>
        {items.length > 0 && (
          <button className="shelf-clear" onClick={onClear}>
            {t("shelf.clear")}
          </button>
        )}
      </div>
      {items.length === 0 ? (
        <div className="shelf-empty">{t("shelf.empty")}</div>
      ) : (
        <div className="shelf-row">
          {items.map((i) => (
            <Tile key={i.path} item={i} onRemove={onRemove} />
          ))}
        </div>
      )}
    </div>
  );
}
