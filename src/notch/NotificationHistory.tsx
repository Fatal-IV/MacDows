// Every notification seen so far, newest first, in the notch.
import { useEffect, useState } from "react";
import { clearNotificationHistory, getNotificationHistory, onNotificationHistoryChanged, type Notice } from "../bridge";
import { useLocale, useT } from "../i18n";

function ago(ms: number, lang: string, t: (k: string) => string): string {
  const diff = Date.now() - ms;
  if (diff < 60_000) return t("history.now");
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)} ${t("history.min")}`;
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)} ${t("history.hour")}`;
  return new Date(ms).toLocaleDateString(lang, { day: "numeric", month: "short" });
}

export function NotificationHistory() {
  const t = useT();
  const lang = useLocale();
  const [items, setItems] = useState<Notice[] | null>(null);

  useEffect(() => {
    const load = () => getNotificationHistory().then(setItems).catch(() => setItems([]));
    load();
    const un = onNotificationHistoryChanged(load);
    const id = setInterval(load, 30_000); // keeps the "5 min" labels fresh
    return () => {
      un.then((f) => f());
      clearInterval(id);
    };
  }, []);

  return (
    <div className="history">
      <div className="shelf-head">
        <span className="shelf-title">{t("history.title")}</span>
        <span className="history-count">{items ? `${items.length} ${t("history.countLabel")}` : ""}</span>
        {items && items.length > 0 && (
          <button className="shelf-clear" onClick={() => clearNotificationHistory().then(() => setItems([])).catch(() => {})}>
            {t("shelf.clear")}
          </button>
        )}
      </div>
      {items && items.length === 0 ? (
        <div className="shelf-empty">{t("history.empty")}</div>
      ) : (
        <div className="history-list">
          {(items ?? []).map((n, i) => (
            <div className="history-item" key={`${n.time_ms}-${i}`}>
              <span className="banner-icon small">{n.icon_png_base64 ? <img src={`data:image/png;base64,${n.icon_png_base64}`} alt="" draggable={false} /> : <span>{n.app.slice(0, 1)}</span>}</span>
              <div className="banner-text">
                <span className="banner-app">{n.app}</span>
                <span className="banner-title">{n.title}</span>
                {n.body && <span className="banner-body">{n.body}</span>}
              </div>
              <span className="history-time">{ago(n.time_ms, lang, t)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
