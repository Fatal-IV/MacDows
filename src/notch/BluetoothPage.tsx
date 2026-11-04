// Paired Bluetooth devices: who is connected, battery level, connect / disconnect.
import { useEffect, useState, type ReactNode } from "react";
import { getBluetoothDevices, quickLaunch, setBluetoothConnection, type BtDevice, type BtKind } from "../bridge";
import { useT } from "../i18n";
import { PairPage } from "./PairPage";

const icon = (children: ReactNode) => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
    {children}
  </svg>
);

const KIND_ICON: Record<BtKind, ReactNode> = {
  audio: icon(<><path d="M4 15v-3a8 8 0 0 1 16 0v3" /><rect x="3" y="14" width="4" height="6" rx="1.6" /><rect x="17" y="14" width="4" height="6" rx="1.6" /></>),
  phone: icon(<><rect x="7" y="3" width="10" height="18" rx="2.4" /><path d="M11 18h2" /></>),
  computer: icon(<><rect x="4" y="5" width="16" height="11" rx="1.8" /><path d="M2.5 19.5h19" /></>),
  input: icon(<><rect x="3" y="7" width="18" height="10" rx="2.4" /><path d="M7 11h.01M10 11h.01M13 11h.01M16 11h.01M8 14h8" /></>),
  other: icon(<path d="M7 7.5l10 9-5 4.2V3.3l5 4.2-10 9" />),
};

function Battery({ percent }: { percent: number }) {
  return (
    <span className="bt-battery" title={`${percent}%`}>
      <svg width="20" height="11" viewBox="0 0 24 12" aria-hidden>
        <rect x="0.75" y="0.75" width="19.5" height="10.5" rx="3" fill="none" stroke="currentColor" strokeWidth="1.5" />
        <rect x="2.5" y="2.5" width={Math.max(1, (percent / 100) * 16)} height="7" rx="1.5" fill="currentColor" />
        <rect x="21.5" y="4" width="2" height="4" rx="1" fill="currentColor" />
      </svg>
      {percent}%
    </span>
  );
}

export function BluetoothPage({ onBack, pairing = true, onEditing }: { onBack: () => void; pairing?: boolean; onEditing?: (on: boolean) => void }) {
  const t = useT();
  const [adding, setAdding] = useState(false);
  const [devices, setDevices] = useState<BtDevice[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [failed, setFailed] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    const load = () => getBluetoothDevices().then((d) => alive && setDevices(d)).catch(() => alive && setDevices([]));
    load();
    const id = setInterval(() => !busy && load(), 5000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [busy]);

  if (adding) return <PairPage onBack={() => setAdding(false)} onEditing={onEditing} />;

  const toggle = async (d: BtDevice) => {
    if (busy) return;
    setBusy(d.address);
    setFailed(null);
    try {
      const next = await setBluetoothConnection(d.address, !d.connected);
      setDevices(next);
      const now = next.find((x) => x.address === d.address);
      // A device that did not change state could not be switched this way.
      if (now && now.connected === d.connected) setFailed(d.address);
    } catch {
      setFailed(d.address);
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="devpage">
      <div className="devpage-head">
        <button className="devpage-back" onClick={onBack} aria-label={t("common.back")}>‹</button>
        <span className="shelf-title">{t("bt.title")}</span>
        <button className="shelf-clear" onClick={() => (pairing ? setAdding(true) : quickLaunch("bluetooth_settings").catch(() => {}))}>{t("bt.add")}</button>
      </div>
      {devices && devices.length === 0 ? (
        <div className="shelf-empty">{t("bt.empty")}</div>
      ) : (
        <div className="history-list">
          {(devices ?? []).map((d) => (
            <div className={`bt-row${d.connected ? " on" : ""}`} key={d.address}>
              <span className="bt-icon">{KIND_ICON[d.kind]}</span>
              <div className="banner-text">
                <span className="banner-title">{d.name}</span>
                <span className="banner-body">{failed === d.address ? t("bt.failed") : d.connected ? t("bt.connected") : t("bt.notConnected")}</span>
              </div>
              {d.battery !== null && <Battery percent={d.battery} />}
              <button className="bt-action" disabled={busy !== null} onClick={() => toggle(d)}>
                {busy === d.address ? t("bt.working") : d.connected ? t("bt.disconnect") : t("bt.connect")}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
