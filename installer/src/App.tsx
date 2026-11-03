import { useEffect, useState } from "react";
import mark from "./assets/mark.svg";
import { closeWindow, installState, launchApp, mockScreen, onDone, onError, onStage, startInstall, type InstallInfo, type StageEvent } from "./api";
import { text, type Key } from "./i18n";
import "./App.css";

type Screen = "ready" | "working" | "done" | "error";

const initialScreen = (): Screen =>
  mockScreen === "working" || mockScreen === "done" || mockScreen === "error" ? mockScreen : "ready";

export default function App() {
  const [info, setInfo] = useState<InstallInfo | null>(null);
  const [screen, setScreen] = useState<Screen>(initialScreen);
  const [stage, setStage] = useState<StageEvent["stage"]>("installing");
  const [percent, setPercent] = useState(mockScreen === "working" ? 60 : 0);
  const [autostart, setAutostart] = useState(false);
  const [error, setError] = useState(mockScreen === "error" ? "The installer stopped with code 2." : "");

  const lang = info?.lang ?? "en";
  const t = (key: Key) => text(lang, key);

  useEffect(() => {
    installState().then(setInfo).catch(() => {});
    const unlisten = [
      onStage((e) => {
        setStage(e.stage);
        setPercent(e.percent);
      }),
      onDone(() => setScreen("done")),
      onError((message) => {
        setError(message);
        setScreen("error");
      }),
    ];
    return () => unlisten.forEach((u) => u.then((f) => f()));
  }, []);

  const install = () => {
    setScreen("working");
    setStage("preparing");
    setPercent(0);
    startInstall(info?.decision === "fresh" ? autostart : undefined).catch((e) => {
      setError(String(e));
      setScreen("error");
    });
  };

  const open = () => {
    launchApp()
      .then(() => closeWindow())
      .catch((e) => {
        setError(String(e));
        setScreen("error");
      });
  };

  const decision = info?.decision ?? "fresh";
  const statusKey: Key = stage === "preparing" ? "preparing" : stage === "installing" ? "installing" : "finishing";

  return (
    <div className="window">
      <div className="card">
        <div className="drag" data-tauri-drag-region />
        <button className="x" onClick={() => closeWindow()} aria-label={t("close")}>
          ✕
        </button>

        {screen === "ready" && (
          <div className="panel">
            <img className="logo" src={mark} alt="" draggable={false} />
            <div className="wordmark">MacDows</div>
            {decision === "fresh" || decision === "older" ? (
              <>
                <div className="muted tight">{t("slogan")}</div>
                {decision === "fresh" && (
                  <button type="button" role="switch" aria-checked={autostart} className="opt" onClick={() => setAutostart((v) => !v)}>
                    <span className="sw" />
                    {t("startWithWindows")}
                  </button>
                )}
                <button className="accent" style={{ marginTop: decision === "fresh" ? 18 : 26 }} onClick={install}>
                  {t("install")}
                </button>
                <div className="muted small" style={{ marginTop: 14 }}>
                  {t("licenseNote")}
                </div>
              </>
            ) : decision === "reinstall" ? (
              <>
                <div className="muted tight">{t("alreadyInstalled")}</div>
                <div className="row" style={{ marginTop: 26 }}>
                  <button className="quiet" onClick={install}>
                    {t("reinstall")}
                  </button>
                  <button className="accent" onClick={open}>
                    {t("open")}
                  </button>
                </div>
              </>
            ) : (
              <>
                <div className="muted tight">
                  {t("newerInstalled")} ({info?.installed_version})
                </div>
                <button className="accent" style={{ marginTop: 26 }} onClick={open}>
                  {t("open")}
                </button>
              </>
            )}
          </div>
        )}

        {screen === "working" && (
          <div className="panel working">
            <img className="logo" src={mark} alt="" draggable={false} />
            <div className="wordmark">MacDows</div>
            <div className="bar">
              <div className="fill" style={{ width: `${percent}%` }} />
            </div>
            <div className="muted small" style={{ marginTop: 10, fontSize: 12 }}>
              {t(statusKey)}
            </div>
          </div>
        )}

        {screen === "done" && (
          <div className="panel done">
            <div className="badge ok">✓</div>
            <div className="wordmark sub">{t("doneTitle")}</div>
            <div className="muted" style={{ marginTop: 6 }}>
              {t("doneText")}
            </div>
            <button className="accent" style={{ marginTop: 24 }} onClick={open}>
              {t("open")}
            </button>
          </div>
        )}

        {screen === "error" && (
          <div className="panel errorpanel">
            <div className="badge bad">!</div>
            <div className="wordmark err">{t("errorTitle")}</div>
            <div className="muted" style={{ marginTop: 8 }}>
              {error}
            </div>
            <div className="row" style={{ marginTop: 22 }}>
              <button className="quiet" onClick={() => closeWindow()}>
                {t("close")}
              </button>
              <button className="accent" onClick={install}>
                {t("retry")}
              </button>
            </div>
            <div className="muted small" style={{ marginTop: 14, fontSize: 10 }}>
              {info?.log_path}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
