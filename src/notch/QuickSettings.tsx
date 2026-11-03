// Wi-Fi, Bluetooth, Do not disturb and Battery saver as glass tiles, the second page of the tools panel.
import { useEffect, useState, type ReactNode } from "react";
import { getQuickSettings, setQuickSetting, type QuickKey, type QuickSettings } from "../bridge";
import { useT } from "../i18n";

const svg = (children: ReactNode) => (
  <svg width="26" height="26" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
    {children}
  </svg>
);

const ICONS: Record<QuickKey, ReactNode> = {
  wifi: svg(
    <>
      <path d="M2.5 9.2a14 14 0 0 1 19 0" />
      <path d="M5.6 12.6a9.4 9.4 0 0 1 12.8 0" />
      <path d="M8.8 16a4.8 4.8 0 0 1 6.4 0" />
      <circle cx="12" cy="19.2" r="1.1" fill="currentColor" stroke="none" />
    </>,
  ),
  bluetooth: svg(<path d="M7 7.5l10 9-5 4.2V3.3l5 4.2-10 9" />),
  dnd: svg(<path d="M20.5 14.6A8.5 8.5 0 0 1 9.4 3.5a8.5 8.5 0 1 0 11.1 11.1Z" />),
  battery_saver: svg(
    <>
      <rect x="3" y="7" width="16" height="10" rx="2.4" />
      <path d="M21.5 10.4v3.2" />
      <path d="M11.4 9.4c-2 .2-3.3 1.6-3.3 3.1 0 1.2.9 2.1 2.1 2.1 1.9 0 2.9-1.9 2.9-3.7" />
    </>,
  ),
};

const ORDER: { key: QuickKey; label: string }[] = [
  { key: "wifi", label: "quick.wifi" },
  { key: "bluetooth", label: "quick.bluetooth" },
  { key: "dnd", label: "quick.dnd" },
  { key: "battery_saver", label: "quick.saver" },
];

export function QuickSettingsPage() {
  const t = useT();
  const [state, setState] = useState<QuickSettings | null>(null);
  const [busy, setBusy] = useState<QuickKey | null>(null);
  const [failed, setFailed] = useState<QuickKey | null>(null);

  useEffect(() => {
    let alive = true;
    const load = () => getQuickSettings().then((s) => alive && setState(s)).catch(() => {});
    load();
    // Windows can change these by itself (airplane mode, the real quick settings), so refresh while the page is open.
    const id = setInterval(load, 4000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  const toggle = async (key: QuickKey) => {
    if (!state || busy || !state[key].available) return;
    setBusy(key);
    setFailed(null);
    try {
      setState(await setQuickSetting(key, !state[key].on));
    } catch {
      setFailed(key);
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="quick">
      {ORDER.map(({ key, label }) => {
        const tile = state?.[key];
        const unavailable = tile ? !tile.available : false;
        const on = !!tile?.on && !unavailable;
        return (
          <button
            key={key}
            className={`glass-tile${on ? " on" : ""}${busy === key ? " busy" : ""}`}
            disabled={!tile || unavailable}
            onClick={() => toggle(key)}
            aria-pressed={on}
          >
            <span className="glass-icon">{ICONS[key]}</span>
            <span className="glass-name">{t(label)}</span>
            <span className="glass-state">{failed === key ? t("quick.error") : unavailable ? t("quick.unavailable") : on ? t("quick.on") : t("quick.off")}</span>
          </button>
        );
      })}
    </div>
  );
}
