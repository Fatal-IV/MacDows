// Windows VPN connections: connect and disconnect with stored credentials. App VPNs (WireGuard, OpenVPN, ...) are only reported.
import { useEffect, useRef, useState } from "react";
import { getVpnState, quickLaunch, vpnConnect, vpnDisconnect, type VpnState } from "../bridge";
import { useT } from "../i18n";
import { announceLink } from "./linkBus";

const SHIELD = (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
    <path d="M12 3l7.5 3v5.5c0 4.6-3.1 8.2-7.5 9.5-4.4-1.3-7.5-4.9-7.5-9.5V6L12 3Z" />
  </svg>
);

export function VpnPage({ onBack }: { onBack: () => void }) {
  const t = useT();
  const [state, setState] = useState<VpnState | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [error, setError] = useState<{ name: string; key: string } | null>(null);
  const busyRef = useRef<string | null>(null);
  busyRef.current = busy;

  useEffect(() => {
    let alive = true;
    const load = () => getVpnState().then((s) => alive && setState(s)).catch(() => alive && setState({ entries: [], app_vpn_active: false }));
    load();
    const id = setInterval(() => !busyRef.current && load(), 4000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  const run = async (name: string, action: () => Promise<unknown>, announce: "connect" | "disconnect") => {
    if (busy) return;
    setBusy(name);
    setError(null);
    try {
      await action();
      announceLink({ kind: "vpn", name, state: announce === "connect" ? "connected" : "disconnected" });
    } catch (e) {
      setError({ name, key: String(e) });
      if (announce === "connect") announceLink({ kind: "vpn", name, state: "failed" });
    } finally {
      setBusy(null);
      getVpnState().then(setState).catch(() => {});
    }
  };

  const line = (name: string, st: string) => {
    if (error?.name === name) return error.key === "credentials" ? t("vpn.credentials") : t("vpn.failed");
    if (busy === name || st === "connecting") return t("vpn.connecting");
    return st === "connected" ? t("vpn.connected") : t("vpn.disconnected");
  };

  const openSettings = () => quickLaunch("vpn_settings").catch(() => {});

  return (
    <div className="devpage">
      <div className="devpage-head">
        <button className="devpage-back" onClick={onBack} aria-label={t("common.back")}>‹</button>
        <span className="shelf-title">{t("vpn.title")}</span>
        <button className="shelf-clear" onClick={openSettings}>{t("vpn.settings")}</button>
      </div>
      {state && state.app_vpn_active && <div className="banner-body">{t("vpn.appOn")}</div>}
      {state && state.entries.length === 0 ? (
        <div className="shelf-empty">{t("vpn.empty")}</div>
      ) : (
        <div className="history-list">
          {(state?.entries ?? []).map((e) => (
            <div className={`bt-row${e.state === "connected" ? " on" : ""}`} key={e.name}>
              <span className="bt-icon">{SHIELD}</span>
              <div className="banner-text">
                <span className="banner-title">{e.name}</span>
                <span className="banner-body">{line(e.name, e.state)}</span>
              </div>
              {error?.name === e.name && error.key === "credentials" && <button className="bt-action" onClick={openSettings}>{t("net.openSettings")}</button>}
              <button className="bt-action" disabled={busy !== null || e.state === "connecting"} onClick={() => run(e.name, () => (e.state === "connected" ? vpnDisconnect(e.name) : vpnConnect(e.name)), e.state === "connected" ? "disconnect" : "connect")}>
                {busy === e.name ? t("net.working") : e.state === "connected" ? t("net.disconnect") : t("net.connect")}
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
