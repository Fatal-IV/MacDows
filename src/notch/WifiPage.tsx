// Wi-Fi networks in range: signal, security, connect to saved or open networks, join secured and hidden ones with a typed password, disconnect, forget.
import { useEffect, useRef, useState } from "react";
import { getWifiState, quickLaunch, wifiConnect, wifiConnectNew, wifiDisconnect, wifiForget, wifiScan, type WifiNetwork, type WifiState } from "../bridge";
import { useT } from "../i18n";
import { canSubmit, errorText, securityLabel, signalBars, type WifiAuth } from "./netLogic";

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

/** Focus and blur handlers for the page's text fields. Moving from one field to the next must not read as "stopped typing". */
function useEditing(onEditing: (on: boolean) => void) {
  const timer = useRef<number | null>(null);
  const active = useRef(false);
  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current);
      if (active.current) onEditing(false);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [],
  );
  return {
    onFocus: () => {
      if (timer.current) clearTimeout(timer.current);
      if (!active.current) {
        active.current = true;
        onEditing(true);
      }
    },
    onBlur: () => {
      if (timer.current) clearTimeout(timer.current);
      timer.current = window.setTimeout(() => {
        if (active.current) {
          active.current = false;
          onEditing(false);
        }
      }, 150);
    },
  };
}

export function WifiPage({ onBack, onEditing = () => {} }: { onBack: () => void; onEditing?: (on: boolean) => void }) {
  const t = useT();
  const field = useEditing(onEditing);
  const [state, setState] = useState<WifiState | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<{ ssid: string; key: string } | null>(null);
  const [confirmForget, setConfirmForget] = useState<string | null>(null);
  // The row whose password is being typed, and the typed value (kept only while the row is open).
  const [joining, setJoining] = useState<string | null>(null);
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  // The hidden-network form.
  const [hiddenOpen, setHiddenOpen] = useState(false);
  const [hSsid, setHSsid] = useState("");
  const [hAuth, setHAuth] = useState<WifiAuth>("wpa2");
  const [hPassword, setHPassword] = useState("");
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
      setJoining(null);
      setPassword("");
      if (hiddenOpen && ssid === hSsid) {
        setHiddenOpen(false);
        setHSsid("");
        setHPassword("");
      }
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

  const submitJoin = (n: WifiNetwork) => {
    const auth: WifiAuth = n.auth === "wpa3" ? "wpa3" : "wpa2";
    if (canSubmit(n.ssid, auth, password)) run(n.ssid, () => wifiConnectNew(n.ssid, password, auth, false));
  };

  const rows = (state?.networks ?? []).map((n) => (
    <div key={n.ssid}>
      <div className={`bt-row${n.connected ? " on" : ""}`}>
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
            ) : n.secured && !n.saved ? (
              <button
                className="bt-action"
                disabled={busy !== null}
                onClick={() => {
                  setJoining(joining === n.ssid ? null : n.ssid);
                  setPassword("");
                  setShowPassword(false);
                  setError(null);
                }}
              >
                {t("net.connect")}
              </button>
            ) : (
              <button className="bt-action" disabled={busy !== null} onClick={() => run(n.ssid, () => wifiConnect(n.ssid))}>
                {busy === n.ssid ? t("net.working") : t("net.connect")}
              </button>
            )}
            {n.saved && !n.enterprise && (
              <button className="bt-action" disabled={busy !== null} onClick={() => setConfirmForget(n.ssid)}>{t("net.forget")}</button>
            )}
          </>
        )}
      </div>
      {joining === n.ssid && (
        <div className="bt-row">
          <input
            className="net-input"
            type={showPassword ? "text" : "password"}
            placeholder={t("net.password")}
            autoFocus
            autoComplete="off"
            spellCheck={false}
            value={password}
            disabled={busy !== null}
            {...field}
            onChange={(e) => setPassword(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") submitJoin(n);
              if (e.key === "Escape") setJoining(null);
            }}
          />
          <button className="bt-action" onClick={() => setShowPassword((v) => !v)}>{showPassword ? t("net.hide") : t("net.show")}</button>
          <button className="bt-action" disabled={busy !== null || !canSubmit(n.ssid, n.auth === "wpa3" ? "wpa3" : "wpa2", password)} onClick={() => submitJoin(n)}>
            {busy === n.ssid ? t("net.working") : t("net.join")}
          </button>
        </div>
      )}
    </div>
  ));

  const hiddenReady = canSubmit(hSsid, hAuth, hPassword);
  const submitHidden = () => hiddenReady && run(hSsid, () => wifiConnectNew(hSsid, hAuth === "open" ? null : hPassword, hAuth, true));

  return (
    <div className="devpage">
      <div className="devpage-head">
        <button className="devpage-back" onClick={onBack} aria-label={t("common.back")}>‹</button>
        <span className="shelf-title">{t("net.wifiTitle")}</span>
        <button className="shelf-clear" onClick={() => setHiddenOpen((v) => !v)}>{t("net.hidden")}</button>
      </div>
      {hiddenOpen && (
        <div className="bt-row">
          <input className="net-input" placeholder={t("net.ssid")} autoComplete="off" spellCheck={false} value={hSsid} disabled={busy !== null} {...field} onChange={(e) => setHSsid(e.target.value)} onKeyDown={(e) => e.key === "Enter" && submitHidden()} />
          <select className="net-input net-select" value={hAuth} disabled={busy !== null} {...field} onChange={(e) => setHAuth(e.target.value as WifiAuth)}>
            <option value="wpa2">{t("net.secWpa2")}</option>
            <option value="wpa3">{t("net.secWpa3")}</option>
            <option value="open">{t("net.secOpen")}</option>
          </select>
          {hAuth !== "open" && (
            <input className="net-input" type="password" placeholder={t("net.password")} autoComplete="off" spellCheck={false} value={hPassword} disabled={busy !== null} {...field} onChange={(e) => setHPassword(e.target.value)} onKeyDown={(e) => e.key === "Enter" && submitHidden()} />
          )}
          <button className="bt-action" disabled={busy !== null || !hiddenReady} onClick={submitHidden}>{busy === hSsid && hSsid ? t("net.working") : t("net.join")}</button>
        </div>
      )}
      {error && hiddenOpen && error.ssid === hSsid && <div className="banner-body">{errorText(error.key, t)}</div>}
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
