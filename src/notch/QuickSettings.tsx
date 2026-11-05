// The quick settings screen of the notch: Wi-Fi, Bluetooth, Do not disturb and Battery saver as wide glass tiles (2 x 2), brightness and volume sliders beside them.
import { useEffect, useRef, useState, type ReactNode } from "react";
import { getBluetoothDevices, getVpnState, vpnConnect, vpnDisconnect, type VpnState, getBrightness, getQuickSettings, getVolume, openMacdowsSettings, quickLaunch, setBrightness, setQuickSetting, setVolume, type BtDevice, type QuickKey, type QuickSettings, type QuickShortcut } from "../bridge";
import { useT } from "../i18n";
import { BluetoothPage } from "./BluetoothPage";
import { WifiPage } from "./WifiPage";
import { VpnPage } from "./VpnPage";
import { vpnIsOn, vpnLabel } from "./netLogic";
import { connectedLabel } from "./bluetoothLabel";

const svg = (children: ReactNode, size = 22) => (
  <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
    {children}
  </svg>
);

const ICONS: Partial<Record<QuickKey, ReactNode>> = {
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

/** The tiles that have a page of their own behind an arrow. */
const DETAIL: QuickKey[] = ["bluetooth"];

const ORDER: { key: QuickKey; label: string }[] = [
  { key: "wifi", label: "quick.wifi" },
  { key: "bluetooth", label: "quick.bluetooth" },
  { key: "dnd", label: "quick.dnd" },
];

/** Round switches in the row under the sliders. */
const SWITCHES: { key: QuickKey; label: string }[] = [
  { key: "battery_saver", label: "quick.saver" },
  { key: "airplane", label: "quick.airplane" },
  { key: "hotspot", label: "quick.hotspot" },
];

export function QuickSettingsScreen({ wifiPage = true, vpn = true, bluetoothPairing = true, onEditing }: { wifiPage?: boolean; vpn?: boolean; bluetoothPairing?: boolean; onEditing?: (on: boolean) => void }) {
  const t = useT();
  const [state, setState] = useState<QuickSettings | null>(null);
  const [busy, setBusy] = useState<QuickKey | null>(null);
  const [failed, setFailed] = useState<QuickKey | null>(null);
  // Wi-Fi and Bluetooth open a page of their own; the arrow on the tile leads there.
  const [page, setPage] = useState<QuickKey | "vpn" | null>(null);
  // The VPN card: Windows VPN connections and whether an app has a tunnel up.
  const [vpnState, setVpnState] = useState<VpnState | null>(null);
  const [vpnBusy, setVpnBusy] = useState(false);
  useEffect(() => {
    if (!vpn || page === "vpn") return;
    let alive = true;
    const load = () => getVpnState().then((s) => alive && setVpnState(s)).catch(() => {});
    load();
    const id = setInterval(load, 5000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [vpn, page]);
  // One Windows VPN: the card switches it on and off. Zero or several: the card opens the page.
  const vpnToggle = async () => {
    const entries = vpnState?.entries ?? [];
    if (entries.length !== 1) return setPage("vpn");
    if (vpnBusy) return;
    setVpnBusy(true);
    try {
      if (entries[0].state === "connected") await vpnDisconnect(entries[0].name);
      else await vpnConnect(entries[0].name);
    } catch {
      setPage("vpn");
    } finally {
      setVpnBusy(false);
      getVpnState().then(setVpnState).catch(() => {});
    }
  };
  // Who is connected, for the Bluetooth tile. Listing devices also reads batteries, so it runs less often than the tiles.
  const [btDevices, setBtDevices] = useState<BtDevice[] | null>(null);
  const btOn = !!state?.bluetooth.on && !!state?.bluetooth.available;

  useEffect(() => {
    if (!btOn || page === "bluetooth") return;
    let alive = true;
    const load = () => getBluetoothDevices().then((d) => alive && setBtDevices(d)).catch(() => {});
    load();
    const id = setInterval(load, 8000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [btOn, page]);

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

  if (page === "bluetooth") return <BluetoothPage onBack={() => setPage(null)} pairing={bluetoothPairing} onEditing={onEditing} />;
  if (page === "vpn" && vpn) return <VpnPage onBack={() => setPage(null)} />;
  if (page === "wifi" && wifiPage) return <WifiPage onBack={() => setPage(null)} onEditing={onEditing} />;

  return (
    <div className="quick">
      <div className={`quick-tiles${vpn ? " has-vpn" : ""}`}>
      {ORDER.map(({ key, label }) => {
        const tile = state?.[key];
        const unavailable = tile ? !tile.available : false;
        const on = !!tile?.on && !unavailable;
        return (
          <div key={key} className={`glass-tile${on ? " on" : ""}${busy === key ? " busy" : ""}${!tile || unavailable ? " disabled" : ""}`}>
            <button className="tile-main" disabled={!tile || unavailable} onClick={() => toggle(key)} aria-pressed={on}>
              <span className="glass-icon">{ICONS[key]}</span>
              <span className="glass-text">
                <span className="glass-name">{t(label)}</span>
                <span className="glass-state">{failed === key ? t("quick.error") : unavailable ? t("quick.unavailable") : on ? (key === "bluetooth" ? connectedLabel(btDevices, t("quick.on")) : t("quick.on")) : t("quick.off")}</span>
              </span>
            </button>
            {(DETAIL.includes(key) || (key === "wifi" && wifiPage)) && (
              <button className="tile-more" aria-label={t("common.more")} title={t("common.more")} onClick={() => setPage(key)}>
                ›
              </button>
            )}
          </div>
        );
      })}
      {vpn && (
        <div className={`glass-tile${vpnIsOn(vpnState) ? " on" : ""}${vpnBusy ? " busy" : ""}`}>
          <button className="tile-main" onClick={vpnToggle}>
            <span className="glass-icon">{svg(<path d="M12 3l7.5 3v5.5c0 4.6-3.1 8.2-7.5 9.5-4.4-1.3-7.5-4.9-7.5-9.5V6L12 3Z" />)}</span>
            <span className="glass-text">
              <span className="glass-name">{t("vpn.title")}</span>
              <span className="glass-state">{vpnBusy ? t("vpn.connecting") : vpnLabel(vpnState, t)}</span>
            </span>
          </button>
          <button className="tile-more" aria-label={t("common.more")} title={t("common.more")} onClick={() => setPage("vpn")}>
            ›
          </button>
        </div>
      )}
      </div>
      <div className="quick-sliders">
        <GlassSlider label="quick.brightness" icon={SUN} read={getBrightness} write={setBrightness} />
        <GlassSlider label="quick.volume" icon={SPEAKER} read={() => getVolume().then((v) => (v ? Math.round(v.level * 100) : null))} write={(p) => setVolume(p / 100).then(() => {})} />
      </div>
      <div className="quick-shortcuts">
        {SWITCHES.map(({ key, label }) => {
          const tile = state?.[key];
          const on = !!tile?.on && !!tile?.available;
          return (
            <button key={key} className={`shortcut${on ? " on" : ""}${busy === key ? " busy" : ""}`} disabled={!tile || !tile.available} title={`${t(label)}: ${on ? t("quick.on") : t("quick.off")}`} aria-label={t(label)} aria-pressed={on} onClick={() => toggle(key)}>
              {SWITCH_ICONS[key]}
            </button>
          );
        })}
        <button className="shortcut" title={t("quick.accessibility")} aria-label={t("quick.accessibility")} onClick={() => quickLaunch("accessibility").catch(() => {})}>
          {ACCESSIBILITY}
        </button>
        <span className="shortcut-sep" />
        {SHORTCUTS.map(({ id, label, icon }) => (
          <button key={id} className="shortcut" title={t(label)} aria-label={t(label)} onClick={() => quickLaunch(id).catch(() => {})}>
            {icon}
          </button>
        ))}
        <button className="shortcut" title={t("quick.macdows")} aria-label={t("quick.macdows")} onClick={() => openMacdowsSettings().catch(() => {})}>
          {svg(<><path d="M4 7h9M17 7h3M4 17h3M11 17h9" /><circle cx="15" cy="7" r="2" /><circle cx="9" cy="17" r="2" /></>, 17)}
        </button>
        <button className="shortcut" title={t("quick.settings")} aria-label={t("quick.settings")} onClick={() => quickLaunch("settings").catch(() => {})}>
          {GEAR}
        </button>
      </div>
    </div>
  );
}

const GEAR = svg(<><circle cx="12" cy="12" r="3" /><path d="M12 2.8v2.6M12 18.6v2.6M2.8 12h2.6M18.6 12h2.6M5.5 5.5l1.8 1.8M16.7 16.7l1.8 1.8M18.5 5.5l-1.8 1.8M7.3 16.7l-1.8 1.8" /></>, 17);
const ACCESSIBILITY = svg(<><circle cx="12" cy="4.6" r="1.7" /><path d="M5 8.5l7 1.4 7-1.4M12 9.9v4.2M12 14.1l-3 6M12 14.1l3 6" /></>, 17);

const SWITCH_ICONS: Record<string, ReactNode> = {
  battery_saver: svg(<><rect x="3" y="7" width="16" height="10" rx="2.4" /><path d="M21.5 10.4v3.2" /><path d="M11.4 9.4c-2 .2-3.3 1.6-3.3 3.1 0 1.2.9 2.1 2.1 2.1 1.9 0 2.9-1.9 2.9-3.7" /></>, 17),
  airplane: svg(<path d="M21 15.5v-1.8l-8-5V4.2a1.5 1.5 0 0 0-3 0v4.5l-8 5v1.8l8-2.3V18l-2 1.5V21l3.5-1 3.5 1v-1.5L13 18v-4.8l8 2.3Z" />, 17),
  hotspot: svg(<><circle cx="12" cy="12" r="1.6" fill="currentColor" stroke="none" /><path d="M8.6 8.6a4.8 4.8 0 0 0 0 6.8M15.4 8.6a4.8 4.8 0 0 1 0 6.8M5.6 5.6a9 9 0 0 0 0 12.8M18.4 5.6a9 9 0 0 1 0 12.8" /></>, 17),
};

const SHORTCUTS: { id: QuickShortcut; label: string; icon: ReactNode }[] = [
  { id: "lock", label: "quick.lock", icon: svg(<><rect x="5" y="11" width="14" height="9.5" rx="2.5" /><path d="M8 11V8a4 4 0 0 1 8 0v3" /></>, 17) },
];

const SUN = svg(
  <>
    <circle cx="12" cy="12" r="4" />
    <path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M18.7 5.3l-1.6 1.6M6.9 17.1l-1.6 1.6" />
  </>,
  18,
);
const SPEAKER = svg(
  <>
    <path d="M4 9.5v5h3.6L12.5 19V5L7.6 9.5H4Z" />
    <path d="M16 9a4.2 4.2 0 0 1 0 6M18.6 6.4a8 8 0 0 1 0 11.2" />
  </>,
  18,
);

/** A glass slider (0 to 100). `read` returning null means this PC has no such control. */
function GlassSlider({ label, icon, read, write }: { label: string; icon: ReactNode; read: () => Promise<number | null>; write: (percent: number) => Promise<void> }) {
  const t = useT();
  const [value, setValue] = useState<number | null | undefined>(undefined); // undefined: still loading
  const dragging = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  useEffect(() => {
    let alive = true;
    const load = () => read().then((v) => alive && !dragging.current && setValue(v)).catch(() => alive && setValue(null));
    load();
    const id = setInterval(load, 3000);
    return () => {
      alive = false;
      clearInterval(id);
      clearTimeout(timer.current);
    };
  }, []);

  // Slider moves are sent at most every 120 ms, and the last one always goes through.
  const change = (v: number) => {
    setValue(v);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => write(v).catch(() => {}), 120);
  };

  const unavailable = value === null;
  const shown = value ?? 0;
  return (
    <div className={`glass-slider${unavailable ? " off" : ""}`}>
      <span className="slider-icon">{icon}</span>
      <div className="slider-body">
        <span className="slider-name">{t(label)}</span>
        <input
          type="range"
          min={0}
          max={100}
          value={shown}
          disabled={unavailable || value === undefined}
          style={{ ["--v" as string]: `${shown}%` }}
          onPointerDown={() => (dragging.current = true)}
          onPointerUp={() => (dragging.current = false)}
          onChange={(e) => change(Number(e.target.value))}
          aria-label={t(label)}
        />
      </div>
      <span className="slider-value">{unavailable ? t("quick.unavailable") : `${shown}`}</span>
    </div>
  );
}
