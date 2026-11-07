import { useEffect, useState } from "react";
import { getMonitors, onMonitorsChanged, setSetting, type MonitorView, type Settings as AppSettings } from "../bridge";
import { monitorCards, setFlag, showSection, type MonitorCard } from "../monitors";
import { Toggle } from "../ui/controls";
import { FoldCard } from "./FoldCard";

/** One card per monitor with an Island and a Dock switch. Not shown with a single monitor. */
export function MonitorsSection({ s, t }: { s: AppSettings; t: (key: string) => string }) {
  const [live, setLive] = useState<MonitorView[]>([]);
  useEffect(() => {
    let alive = true;
    const load = () => getMonitors().then((m) => alive && setLive(m)).catch(() => {});
    load();
    const un = onMonitorsChanged(load);
    return () => {
      alive = false;
      un.then((f) => f());
    };
  }, [s.monitors]);

  const cards = monitorCards(live, s.monitors);
  if (!showSection(cards)) return null;
  const change = (card: MonitorCard, flag: "island" | "dock", on: boolean) => setSetting("monitors", setFlag(s.monitors, card, flag, on)).catch(() => {});

  return (
    <FoldCard title={t("monitors.title")}>
      <div className="island-style">
        <div className="ui-row-text">
          <div className="ui-row-desc">{t("monitors.help")}</div>
        </div>
        <div className="monitor-cards">
          {cards.map((card, i) => (
            <div key={card.key} className={`monitor-card${card.connected ? "" : " away"}`}>
              <span className="monitor" aria-hidden>
                <span className="monitor-screen">
                  <span className="monitor-island" style={{ opacity: card.island ? 1 : 0.15 }} />
                  <span className="monitor-dock" style={{ opacity: card.dock ? 1 : 0.15 }} />
                </span>
                <span className="monitor-stand" />
              </span>
              <span className="island-style-name">
                {i + 1}
                {card.connected ? ` · ${card.width}×${card.height}` : ""}
                {card.primary ? ` · ${t("monitors.primary")}` : ""}
              </span>
              {!card.connected && <span className="island-style-caption">{t("monitors.notConnected")}</span>}
              <label className="monitor-flag">
                <span>{t("monitors.island")}</span>
                <Toggle label={`${t("monitors.island")} ${i + 1}`} checked={card.island} onChange={(on) => change(card, "island", on)} />
              </label>
              <label className="monitor-flag">
                <span>{t("monitors.dock")}</span>
                <Toggle label={`${t("monitors.dock")} ${i + 1}`} checked={card.dock} onChange={(on) => change(card, "dock", on)} />
              </label>
            </div>
          ))}
        </div>
      </div>
    </FoldCard>
  );
}
