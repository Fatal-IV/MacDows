import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useT } from "../i18n";
import "./windowcontrols.css";

/**
 * Window management for a window without a title bar: three small buttons in the top-right corner (minimize,
 * maximize or restore, close), and an invisible strip along the top edge to drag the window by.
 */
export default function WindowControls() {
  const t = useT();
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    const sync = () => win.isMaximized().then(setMaximized).catch(() => {});
    sync();
    const un = win.onResized(sync);
    return () => {
      un.then((f) => f());
    };
  }, []);

  const win = getCurrentWindow();
  const toggle = maximized ? t("window.restore") : t("window.maximize");
  return (
    <>
      <div className="drag-strip" data-tauri-drag-region />
      <div className="window-controls" role="group" aria-label={t("window.controls")}>
        <button type="button" aria-label={t("window.minimize")} title={t("window.minimize")} onClick={() => win.minimize()}>
          <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden>
            <path d="M2 6h8" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </button>
        <button type="button" aria-label={toggle} title={toggle} onClick={() => win.toggleMaximize()}>
          {maximized ? (
            <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden>
              <rect x="2.5" y="4" width="5.5" height="5.5" rx="1" fill="none" stroke="currentColor" strokeWidth="1.2" />
              <path d="M4.5 4V3.2a.7.7 0 0 1 .7-.7h3.6a.7.7 0 0 1 .7.7v3.6a.7.7 0 0 1-.7.7H8" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
            </svg>
          ) : (
            <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden>
              <rect x="2.5" y="2.5" width="7" height="7" rx="1.2" fill="none" stroke="currentColor" strokeWidth="1.2" />
            </svg>
          )}
        </button>
        <button type="button" className="close" aria-label={t("window.close")} title={t("window.close")} onClick={() => win.close()}>
          <svg viewBox="0 0 12 12" width="12" height="12" aria-hidden>
            <path d="M2.5 2.5l7 7M9.5 2.5l-7 7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
        </button>
      </div>
    </>
  );
}
