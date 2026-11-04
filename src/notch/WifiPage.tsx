// Wi-Fi networks in range: signal, security, connect to saved or open networks, disconnect, forget.
import { useEffect, useRef, useState } from "react";
import { getWifiState, quickLaunch, wifiConnect, wifiDisconnect, wifiForget, wifiScan, type WifiNetwork, type WifiState } from "../bridge";
import { useT } from "../i18n";
import { errorText, securityLabel, signalBars } from "./netLogic";

function Bars({ signal }: { signal: number }) {
  const on = signalBars(signal);
  return (
    <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden>
      {[0, 1, 2, 3].map((i) => (
        <rect key={i} x={3 + i * 5} y={18 - (i + 1) * 4} width="3.4" height={(i + 1) * 4} rx="1" fill="currentColor" opacity={i < on ? 1 : 0.25} />
      ))}
    </svg>
  );
}

export function WifiPage({ onBack }: { onBack: () => void }) {
  const t = useT();
  const [state, setState] = useState<WifiState | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<{ ssid: string; key: string } | null>(null);
  const [confirmForget, setConfirmForget] = useState<string | null>(null);
  const busyRef = useRef<string | null>(null);
  busyRef.current = busy;

  useEffect(() => {
    let alive = true;
    const load = () => getWifiState().then((s) => alive && setState(s)).catch(() => alive && setState({ available: false, enabled: false, networks: [] }));
    wifiScan().catch(() => {});
    load();
    // A scan takes a few seconds: ask again, then read the result on every tick (not while a connect is running).
    const id = setInterval(() => {
      if (busyRef.current) return;
      wifiScan().catch(() => {});
      load();
    }, 5000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  const run = async (ssid: string, action: () => Promise<unknown>) => {
    if (busy) return;
    setBusy(ssid);
    setError(null);
    setConfirmForget(null);
    try {
      await action();
    } catch (e) {
      setError({ ssid, key: String(e) });
    } finally {
      setBusy(null);
      getWifiState().then(setState).catch(() => {});
    }
  };

  const stateLine = (n: WifiNetwork) => {
    if (error?.ssid === n.ssid) return errorText(error.key, t);
    if (busy === n.ssid) return t("net.working");
    if (n.connected) return t("net.connected");
    return n.saved ? t("net.saved") : securityLabel(n, t);
  };

  const rows = (state?.networks ?? []).map((n) => (
    <div className={`bt-row${n.connected ? " on" : ""}`} key={n.ssid}>
      <span className="bt-icon"><Bars signal={n.signal} /></span>
      <div className="banner-text">
        <span className="banner-title">{n.ssid}</span>
        <span className="banner-body">{stateLine(n)}</span>
      </div>
      {confirmForget === n.ssid ? (
        <>
          <span className="banner-body">{t("net.forgetAsk")}</span>
          <button className="bt-action" onClick={() => run(n.ssid, () => wifiForget(n.ssid))}>{t("net.yes")}</button>
          <button className="bt-action" onClick={() => setConfirmForget(null)}>{t("net.cancel")}</button>
        </>
      ) : (
        <>
          {n.enterprise ? (
            <button className="bt-action" onClick={() => quickLaunch("wifi_settings").catch(() => {})}>{t("net.openSettings")}</button>
          ) : n.connected ? (
            <button className="bt-action" disabled={busy !== null} onClick={() => run(n.ssid, () => wifiDisconnect())}>{t("net.disconnect")}</button>
          ) : (
            <button className="bt-action" disabled={busy !== null || (n.secured && !n.saved)} title={n.secured && !n.saved ? t("net.needsPassword") : undefined} onClick={() => run(n.ssid, () => wifiConnect(n.ssid))}>
              {busy === n.ssid ? t("net.working") : t("net.connect")}
            </button>
          )}
          {n.saved && !n.enterprise && (
            <button className="bt-action" disabled={busy !== null} onClick={() => setConfirmForget(n.ssid)}>{t("net.forget")}</button>
          )}
        </>
      )}
    </div>
  ));

  return (
    <div className="devpage">
      <div className="devpage-head">
        <button className="devpage-back" onClick={onBack} aria-label={t("common.back")}>‹</button>
        <span className="shelf-title">{t("net.wifiTitle")}</span>
      </div>
      {state === null ? null : !state.available ? (
        <div className="shelf-empty">{t("net.noAdapter")}</div>
      ) : !state.enabled ? (
        <div className="shelf-empty">{t("net.wifiOff")}</div>
      ) : state.networks.length === 0 ? (
        <div className="shelf-empty">{t("net.noNetworks")}</div>
      ) : (
        <div className="history-list">{rows}</div>
      )}
    </div>
  );
}
