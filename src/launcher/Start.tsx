import { useEffect, useMemo, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { initTheme } from "../appearance/theme";
import { useT } from "../i18n";
import { Icon } from "./icons";
import { closePopup, iconSrc, launchApp, listApps, openLocation, powerAction, startState, togglePin, type AppEntry, type StartState } from "./api";
import { useCloseOnBlur } from "./usePopup";
import "./Launcher.css";

const SHORTCUTS = ["settings", "explorer", "documents", "downloads", "pictures"] as const;
const POWER = ["sleep", "restart", "shutdown", "lock", "signout"] as const;
// Actions that end the session: they ask once more before running.
const NEEDS_CONFIRM = new Set<string>(["restart", "shutdown", "signout"]);

function AppIcon({ app, size }: { app: AppEntry; size: number }) {
  const src = iconSrc(app);
  return src ? (
    <img src={src} width={size} height={size} alt="" draggable={false} />
  ) : (
    <span className="app-fallback" style={{ width: size, height: size }}>{app.name.slice(0, 1).toUpperCase()}</span>
  );
}

function initials(name: string) {
  return name.split(/\s+/).filter(Boolean).slice(0, 2).map((p) => p[0].toUpperCase()).join("") || "?";
}

/** MacDows's own Start menu: pinned apps, recently used apps, every app, places and the power buttons. */
export default function Start() {
  const t = useT();
  const [apps, setApps] = useState<AppEntry[]>([]);
  const [state, setState] = useState<StartState | null>(null);
  const [allApps, setAllApps] = useState(false);
  const [powerOpen, setPowerOpen] = useState(false);
  const [confirm, setConfirm] = useState<string | null>(null);

  useEffect(() => initTheme({ backdrop: false }), []);
  useCloseOnBlur("start");

  const refreshState = () => startState().then(setState).catch(() => {});
  useEffect(() => {
    listApps().then(setApps).catch(() => setApps([]));
    refreshState();
  }, []);

  const byId = useMemo(() => new Map(apps.map((a) => [a.id, a])), [apps]);
  const pick = (ids: string[]) => ids.map((id) => byId.get(id)).filter((a): a is AppEntry => !!a);
  const pinned = pick(state?.pinned ?? []);
  const recent = pick(state?.recent ?? []);

  const open = (app: AppEntry) => {
    launchApp(app.id);
    closePopup("start");
  };
  const pin = (app: AppEntry) => togglePin(app.id).then(refreshState);
  const places = (kind: string) => {
    openLocation(kind);
    closePopup("start");
  };
  const power = (kind: string) => {
    if (NEEDS_CONFIRM.has(kind) && confirm !== kind) {
      setConfirm(kind);
      return;
    }
    powerAction(kind);
    closePopup("start");
  };

  const sections = useMemo(() => {
    const groups = new Map<string, AppEntry[]>();
    for (const app of apps) {
      const first = app.name.trim().charAt(0).toUpperCase();
      const key = /\p{L}/u.test(first) ? first : "#";
      groups.set(key, [...(groups.get(key) ?? []), app]);
    }
    return [...groups.entries()].sort(([a], [b]) => (a === "#" ? -1 : b === "#" ? 1 : a.localeCompare(b)));
  }, [apps]);

  return (
    <motion.div
      className="popup-panel start"
      initial={{ opacity: 0, y: 14, scale: 0.98 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      transition={{ type: "spring", stiffness: 420, damping: 34 }}
      onClick={() => setPowerOpen(false)}
    >
      <div className="start-body">
        <AnimatePresence mode="wait" initial={false}>
          {allApps ? (
            <motion.section key="all" className="start-section" initial={{ opacity: 0, x: 16 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: 16 }} transition={{ duration: 0.12 }}>
              <div className="section-head">
                <h2>{t("start.allApps")}</h2>
                <button className="link" onClick={() => setAllApps(false)}>
                  <Icon name="chevronLeft" size={14} />
                  {t("start.back")}
                </button>
              </div>
              <div className="all-list">
                {sections.map(([letter, items]) => (
                  <div key={letter}>
                    <div className="letter">{letter}</div>
                    {items.map((app) => (
                      <div key={app.id} className="list-item" onClick={() => open(app)}>
                        <AppIcon app={app} size={28} />
                        <span className="name">{app.name}</span>
                        <button
                          className={`pin-btn${state?.pinned.includes(app.id) ? " on" : ""}`}
                          title={t("start.pin")}
                          onClick={(e) => {
                            e.stopPropagation();
                            pin(app);
                          }}
                        >
                          <Icon name="pin" size={15} />
                        </button>
                      </div>
                    ))}
                  </div>
                ))}
              </div>
            </motion.section>
          ) : (
            <motion.div key="home" className="start-home" initial={{ opacity: 0, x: -16 }} animate={{ opacity: 1, x: 0 }} exit={{ opacity: 0, x: -16 }} transition={{ duration: 0.12 }}>
              <section className="start-section">
                <div className="section-head">
                  <h2>{t("start.pinned")}</h2>
                  <button className="link" onClick={() => setAllApps(true)}>
                    {t("start.allApps")}
                    <Icon name="chevronRight" size={14} />
                  </button>
                </div>
                {pinned.length === 0 ? (
                  <p className="empty">{t("start.noPinned")}</p>
                ) : (
                  <div className="pinned-grid">
                    {pinned.map((app) => (
                      <motion.button
                        key={app.id}
                        className="tile"
                        title={app.name}
                        whileHover={{ scale: 1.05 }}
                        whileTap={{ scale: 0.96 }}
                        onClick={() => open(app)}
                        onContextMenu={(e) => {
                          e.preventDefault();
                          pin(app);
                        }}
                      >
                        <AppIcon app={app} size={38} />
                        <span>{app.name}</span>
                      </motion.button>
                    ))}
                  </div>
                )}
              </section>
              <section className="start-section">
                <div className="section-head">
                  <h2>{t("start.recommended")}</h2>
                </div>
                {recent.length === 0 ? (
                  <p className="empty">{t("start.noRecent")}</p>
                ) : (
                  <div className="recent-grid">
                    {recent.map((app) => (
                      <button
                        key={app.id}
                        className="list-item"
                        onClick={() => open(app)}
                        onContextMenu={(e) => {
                          e.preventDefault();
                          pin(app);
                        }}
                      >
                        <AppIcon app={app} size={30} />
                        <span className="name">{app.name}</span>
                      </button>
                    ))}
                  </div>
                )}
              </section>
            </motion.div>
          )}
        </AnimatePresence>
      </div>

      <footer className="start-footer">
        <div className="user">
          <span className="avatar">{initials(state?.user ?? "")}</span>
          <span className="user-name">{state?.user}</span>
        </div>
        <div className="footer-buttons">
          {SHORTCUTS.map((kind) => (
            <button key={kind} className="icon-btn" title={t(`start.place.${kind}`)} aria-label={t(`start.place.${kind}`)} onClick={() => places(kind)}>
              <Icon name={kind} />
            </button>
          ))}
          <span className="vsep" />
          <div className="power-wrap">
            <button
              className={`icon-btn${powerOpen ? " active" : ""}`}
              title={t("power.title")}
              aria-label={t("power.title")}
              onClick={(e) => {
                e.stopPropagation();
                setConfirm(null);
                setPowerOpen((v) => !v);
              }}
            >
              <Icon name="power" />
            </button>
            <AnimatePresence>
              {powerOpen && (
                <motion.div
                  className="power-menu"
                  initial={{ opacity: 0, y: 6, scale: 0.97 }}
                  animate={{ opacity: 1, y: 0, scale: 1 }}
                  exit={{ opacity: 0, y: 6, scale: 0.97 }}
                  transition={{ duration: 0.12 }}
                  onClick={(e) => e.stopPropagation()}
                >
                  {POWER.map((kind) => (
                    <button key={kind} className={confirm === kind ? "danger" : ""} onClick={() => power(kind)}>
                      {confirm === kind ? t("power.confirm") : t(`power.${kind}`)}
                    </button>
                  ))}
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>
      </footer>
    </motion.div>
  );
}
