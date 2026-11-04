import { useEffect, useMemo, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { closeAddAppPanel, getCatalog, onPinnedChanged, pinCatalogApp, type CatalogApp } from "../bridge";
import { initTheme } from "../appearance/theme";
import { useT } from "../i18n";
import "./AddApp.css";

// A blur that fires while the window is still being focused would close it at once.
const BLUR_GRACE_MS = 400;

/** The panel that opens above the dock from "Add app": a searchable list of installed apps; a click pins one. */
export default function AddApp() {
  const t = useT();
  const [apps, setApps] = useState<CatalogApp[] | null>(null);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => initTheme({ backdrop: true }), []);

  const load = () => getCatalog().then(setApps).catch(() => setApps([]));
  useEffect(() => {
    load();
    input.current?.focus();
    // Pinning takes a few seconds to settle (Windows is asked first), so the list is reloaded when the pins change.
    const un = onPinnedChanged(() => {
      setBusy(null);
      load();
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  useEffect(() => {
    const opened = Date.now();
    const close = () => closeAddAppPanel().catch(() => {});
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

  const q = query.trim().toLowerCase();
  const visible = useMemo(() => (apps ?? []).filter((a) => q === "" || a.name.toLowerCase().includes(q)), [apps, q]);
  const running = visible.filter((a) => a.running);
  const rest = visible.filter((a) => !a.running);

  const pin = (a: CatalogApp) => {
    if (a.pinned || busy) return;
    setBusy(a.id);
    pinCatalogApp(a.id).catch(() => setBusy(null));
    // The settle event ends "busy"; this is only the safety net.
    setTimeout(() => setBusy(null), 6000);
  };

  const row = (a: CatalogApp) => (
    <button key={a.id} className={`addapp-row${a.pinned ? " pinned" : ""}${busy === a.id ? " busy" : ""}`} disabled={a.pinned} onClick={() => pin(a)} title={a.name}>
      {a.icon_png_base64 ? <img src={`data:image/png;base64,${a.icon_png_base64}`} alt="" draggable={false} /> : <span className="addapp-fallback">{a.name.slice(0, 1).toUpperCase()}</span>}
      <span className="addapp-name">{a.name}</span>
      {a.pinned && <span className="addapp-check">✓</span>}
    </button>
  );

  return (
    <div className="addapp-panel" onContextMenu={(e) => e.preventDefault()}>
      <input ref={input} className="addapp-search" value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("addapp.search")} spellCheck={false} />
      <div className="addapp-list">
        {apps === null && <div className="addapp-empty">{t("addapp.loading")}</div>}
        {apps !== null && visible.length === 0 && <div className="addapp-empty">{t("addapp.none")}</div>}
        {running.length > 0 && <div className="addapp-section">{t("addapp.running")}</div>}
        {running.map(row)}
        {rest.length > 0 && <div className="addapp-section">{t("addapp.all")}</div>}
        {rest.map(row)}
      </div>
    </div>
  );
}
