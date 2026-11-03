import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import { invoke } from "@tauri-apps/api/core";
import { initTheme } from "../appearance/theme";
import { useT } from "../i18n";
import { Icon } from "./icons";
import { closePopup, iconSrc, launchApp, type AppEntry } from "./api";
import { useCloseOnBlur } from "./usePopup";
import "./Launcher.css";

interface SettingHit {
  page: string;
  name: string;
}
interface FileHit {
  path: string;
  name: string;
  folder: string;
  is_dir: boolean;
}
interface Results {
  calc: string | null;
  apps: AppEntry[];
  settings: SettingHit[];
  files: FileHit[];
}

interface Row {
  key: string;
  group: string;
  title: string;
  subtitle?: string;
  icon: ReactNode;
  run: () => void;
}

const EMPTY: Results = { calc: null, apps: [], settings: [], files: [] };
const DEBOUNCE_MS = 90;

const tile = (name: Parameters<typeof Icon>[0]["name"]) => (
  <span className="result-icon">
    <Icon name={name} size={20} />
  </span>
);

/** MacDows's own search: apps, Windows settings, files and folders, a calculator and web search in one box. */
export default function Search() {
  const t = useT();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<Results>(EMPTY);
  const [selected, setSelected] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);

  useEffect(() => initTheme({ backdrop: false }), []);
  useCloseOnBlur("search");
  useEffect(() => input.current?.focus(), []);

  // Search a moment after typing stops; a stale answer never replaces a newer one.
  useEffect(() => {
    let current = true;
    const text = query.trim();
    if (!text) {
      setResults(EMPTY);
      return;
    }
    const id = setTimeout(() => {
      invoke<Results>("search_all", { query: text })
        .then((r) => current && setResults(r))
        .catch(() => current && setResults(EMPTY));
    }, DEBOUNCE_MS);
    return () => {
      current = false;
      clearTimeout(id);
    };
  }, [query]);

  const done = (fn: () => unknown) => () => {
    fn();
    closePopup("search");
  };

  const rows: Row[] = useMemo(() => {
    const text = query.trim();
    const out: Row[] = [];
    if (results.calc !== null) {
      const value = results.calc;
      out.push({
        key: "calc",
        group: t("search.calculator"),
        title: `= ${value}`,
        subtitle: t("search.copyResult"),
        icon: tile("calculator"),
        run: done(() => navigator.clipboard?.writeText(value)),
      });
    }
    for (const app of results.apps) {
      const src = iconSrc(app);
      out.push({
        key: `app:${app.id}`,
        group: t("search.apps"),
        title: app.name,
        icon: src ? <img className="result-img" src={src} alt="" draggable={false} /> : tile("search"),
        run: done(() => launchApp(app.id)),
      });
    }
    for (const s of results.settings) {
      out.push({
        key: `set:${s.page}`,
        group: t("search.settings"),
        title: s.name,
        subtitle: t("search.windowsSettings"),
        icon: tile("settings"),
        run: done(() => invoke("open_setting", { page: s.page })),
      });
    }
    for (const f of results.files) {
      out.push({
        key: `file:${f.path}`,
        group: t("search.files"),
        title: f.name,
        subtitle: f.folder,
        icon: tile(f.is_dir ? "explorer" : "documents"),
        run: done(() => invoke("open_file", { path: f.path })),
      });
    }
    if (text) {
      out.push({
        key: "web",
        group: t("search.web"),
        title: t("search.webFor").replace("{q}", text),
        icon: tile("search"),
        run: done(() => invoke("open_web_search", { query: text })),
      });
    }
    return out;
    // `done` and `t` are stable enough for a result list that is rebuilt whenever results change.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [results, query]);

  // The selection starts on the first row again whenever the list changes.
  useEffect(() => setSelected(0), [rows.length, query]);
  useEffect(() => {
    list.current?.querySelector<HTMLElement>(`[data-index="${selected}"]`)?.scrollIntoView({ block: "nearest" });
  }, [selected]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setSelected((s) => Math.min(s + 1, Math.max(rows.length - 1, 0)));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      rows[selected]?.run();
    }
  };

  return (
    <motion.div
      className="popup-panel search"
      initial={{ opacity: 0, y: 14, scale: 0.98 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      transition={{ type: "spring", stiffness: 420, damping: 34 }}
    >
      <div className="search-results" ref={list}>
        {rows.length === 0 ? (
          <p className="search-hint">{query.trim() ? t("search.nothing") : t("search.hint")}</p>
        ) : (
          rows.map((row, i) => (
            <div key={row.key}>
              {(i === 0 || rows[i - 1].group !== row.group) && <div className="letter">{row.group}</div>}
              <div
                data-index={i}
                className={`result${i === selected ? " selected" : ""}${row.key === "calc" ? " calc" : ""}`}
                onMouseMove={() => setSelected(i)}
                onClick={row.run}
              >
                {row.icon}
                <div className="result-text">
                  <span className="result-title">{row.title}</span>
                  {row.subtitle && <span className="result-sub">{row.subtitle}</span>}
                </div>
                {i === selected && <span className="enter-hint">{t("search.enter")}</span>}
              </div>
            </div>
          ))
        )}
      </div>
      <div className="search-bar">
        <Icon name="search" size={18} />
        <input ref={input} value={query} placeholder={t("search.placeholder")} onChange={(e) => setQuery(e.target.value)} onKeyDown={onKeyDown} spellCheck={false} autoComplete="off" />
      </div>
    </motion.div>
  );
}
