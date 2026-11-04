// Nearby Bluetooth devices that are not paired yet, and the pairing itself: a plain yes, a code to compare, or a PIN to type.
import { useEffect, useRef, useState } from "react";
import { btPair, btPairingAnswer, btScanStart, btScanStop, onBtFound, onBtLost, onBtPaired, onBtPairingRequest, type BtPairingRequest } from "../bridge";
import { useT } from "../i18n";

interface Nearby {
  id: string;
  name: string;
}

export function PairPage({ onBack, onEditing = () => {} }: { onBack: () => void; onEditing?: (on: boolean) => void }) {
  const t = useT();
  const [devices, setDevices] = useState<Nearby[]>([]);
  const [pairing, setPairing] = useState<string | null>(null);
  const [request, setRequest] = useState<BtPairingRequest | null>(null);
  const [failed, setFailed] = useState<{ id: string; key: string } | null>(null);
  const [pin, setPin] = useState("");
  const typing = useRef(false);

  useEffect(() => {
    let alive = true;
    const offs: Array<Promise<() => void>> = [
      onBtFound((d) => alive && setDevices((list) => (list.some((x) => x.id === d.id) ? list.map((x) => (x.id === d.id ? { ...x, name: d.name } : x)) : [...list, { id: d.id, name: d.name }]))),
      onBtLost((d) => alive && setDevices((list) => list.filter((x) => x.id !== d.id))),
      onBtPaired((d) => alive && setDevices((list) => list.filter((x) => x.id !== d.id))),
      onBtPairingRequest((r) => alive && setRequest(r)),
    ];
    btScanStart().catch(() => {});
    return () => {
      alive = false;
      offs.forEach((p) => p.then((off) => off()).catch(() => {}));
      btScanStop().catch(() => {});
      if (typing.current) onEditing(false);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const start = async (d: Nearby) => {
    if (pairing) return;
    setPairing(d.id);
    setFailed(null);
    setPin("");
    try {
      await btPair(d.id);
    } catch (e) {
      setFailed({ id: d.id, key: String(e) });
    } finally {
      setPairing(null);
      setRequest(null);
      if (typing.current) {
        typing.current = false;
        onEditing(false);
      }
    }
  };

  const answer = (accept: boolean) => {
    btPairingAnswer(accept, request?.kind === "provide_pin" ? pin : null).catch(() => {});
    setRequest(null);
    if (typing.current) {
      typing.current = false;
      onEditing(false);
    }
  };

  const status = (d: Nearby) => {
    if (failed?.id === d.id) return failed.key === "rejected" ? t("pair.rejected") : t("pair.failed");
    if (pairing === d.id) return request?.id === d.id ? t(`pair.ask.${request.kind}`) : t("pair.pairing");
    return t("pair.ready");
  };

  return (
    <div className="devpage">
      <div className="devpage-head">
        <button className="devpage-back" onClick={onBack} aria-label={t("common.back")}>‹</button>
        <span className="shelf-title">{t("pair.title")}</span>
      </div>
      {devices.length === 0 ? (
        <div className="shelf-empty">{t("pair.searching")}</div>
      ) : (
        <div className="history-list">
          {devices.map((d) => {
            const asking = request && request.id === d.id && pairing === d.id;
            return (
              <div className="bt-row" key={d.id}>
                <div className="banner-text">
                  <span className="banner-title">{d.name}</span>
                  <span className="banner-body">{status(d)}{asking && request.pin ? `: ${request.pin}` : ""}</span>
                </div>
                {asking && request.kind === "provide_pin" && (
                  <input
                    className="net-input"
                    placeholder={t("pair.pin")}
                    autoFocus
                    autoComplete="off"
                    inputMode="numeric"
                    value={pin}
                    onFocus={() => {
                      typing.current = true;
                      onEditing(true);
                    }}
                    onChange={(e) => setPin(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && pin) answer(true);
                      if (e.key === "Escape") answer(false);
                    }}
                  />
                )}
                {asking ? (
                  <>
                    <button className="bt-action" disabled={request.kind === "provide_pin" && !pin} onClick={() => answer(true)}>
                      {request.kind === "confirm_pin" ? t("net.yes") : t("pair.pair")}
                    </button>
                    <button className="bt-action" onClick={() => answer(false)}>{t("net.cancel")}</button>
                  </>
                ) : (
                  <button className="bt-action" disabled={pairing !== null} onClick={() => start(d)}>
                    {pairing === d.id ? t("net.working") : t("pair.pair")}
                  </button>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
