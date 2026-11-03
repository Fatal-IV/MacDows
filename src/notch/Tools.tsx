// The calendar and the timer / stopwatch that the notch shows besides the media panel.
import { useEffect, useMemo, useState } from "react";
import { useLocale, useT } from "../i18n";
import { QuickSettingsPage } from "./QuickSettings";

export type TimerMode = "timer" | "stopwatch";

export interface Timer {
  mode: TimerMode;
  /** Milliseconds shown: what is left (timer) or what has passed (stopwatch). */
  ms: number;
  running: boolean;
  setMode: (m: TimerMode) => void;
  toggle: () => void;
  reset: () => void;
  /** Sets the timer to this many milliseconds and starts it. */
  start: (ms: number) => void;
}

/** "12" is 12 minutes, "1:30" is 1 hour 30 minutes, "0:45" is 45 minutes. */
export function parseDuration(text: string): number | null {
  const t = text.trim();
  const m = /^(\d+)(?::(\d{1,2}))?$/.exec(t);
  if (!m) return null;
  const ms = m[2] === undefined ? Number(m[1]) * 60_000 : (Number(m[1]) * 60 + Number(m[2])) * 60_000;
  return ms > 0 ? ms : null;
}

const clock = (ms: number, up: boolean) => {
  const s = up ? Math.floor(ms / 1000) : Math.ceil(ms / 1000);
  const h = Math.floor(s / 3600);
  const mm = Math.floor((s % 3600) / 60);
  const ss = String(s % 60).padStart(2, "0");
  return h ? `${h}:${String(mm).padStart(2, "0")}:${ss}` : `${mm}:${ss}`;
};

/** Lives in the notch itself so the timer keeps running while the panel is closed. */
export function useTimer(): Timer {
  const [mode, setModeState] = useState<TimerMode>("timer");
  // Timer: `left` is what remained when it last stopped. Stopwatch: `spent` is what had passed.
  const [left, setLeft] = useState(0);
  const [spent, setSpent] = useState(0);
  const [since, setSince] = useState<number | null>(null);
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    if (since === null) return;
    const id = setInterval(() => setNow(Date.now()), 200);
    return () => clearInterval(id);
  }, [since]);

  const run = since === null ? 0 : Math.max(0, now - since);
  const ms = mode === "timer" ? Math.max(0, left - run) : spent + run;

  // A timer that reaches zero stops there.
  useEffect(() => {
    if (mode === "timer" && since !== null && left - run <= 0) {
      setSince(null);
      setLeft(0);
    }
  }, [mode, since, left, run]);

  return {
    mode,
    ms,
    running: since !== null,
    setMode: (m) => {
      setSince(null);
      setModeState(m);
    },
    toggle: () => {
      if (since !== null) {
        if (mode === "timer") setLeft(ms);
        else setSpent(ms);
        setSince(null);
      } else if (mode === "stopwatch" || left > 0) {
        setNow(Date.now());
        setSince(Date.now());
      }
    },
    reset: () => {
      setSince(null);
      setLeft(0);
      setSpent(0);
    },
    start: (d) => {
      setModeState("timer");
      setLeft(d);
      setNow(Date.now());
      setSince(Date.now());
    },
  };
}

function Calendar() {
  const [offset, setOffset] = useState(0);
  const today = new Date();
  const first = new Date(today.getFullYear(), today.getMonth() + offset, 1);
  const lang = useLocale();
  const title = first.toLocaleDateString(lang, { month: "long", year: "numeric" });
  const cells = useMemo(() => {
    const lead = (first.getDay() + 6) % 7; // weeks start on Monday
    const days = new Date(first.getFullYear(), first.getMonth() + 1, 0).getDate();
    return [...Array(lead).fill(null), ...Array.from({ length: days }, (_, i) => i + 1)];
  }, [first.getFullYear(), first.getMonth()]);
  const names = useMemo(() => Array.from({ length: 7 }, (_, i) => new Date(2024, 0, 1 + i).toLocaleDateString(lang, { weekday: "narrow" })), [lang]);
  const isToday = (d: number) => offset === 0 && d === today.getDate();
  return (
    <div className="calendar">
      <div className="cal-head">
        <button className="cal-title" onClick={() => setOffset(0)}>{title}</button>
        <span className="cal-nav">
          <button onClick={() => setOffset(offset - 1)} aria-label="‹">‹</button>
          <button onClick={() => setOffset(offset + 1)} aria-label="›">›</button>
        </span>
      </div>
      <div className="cal-grid">
        {names.map((n, i) => <span key={`n${i}`} className="cal-name">{n}</span>)}
        {cells.map((d, i) => <span key={i} className={`cal-day${d && isToday(d) ? " today" : ""}`}>{d ?? ""}</span>)}
      </div>
    </div>
  );
}

const PRESETS = [5, 15, 25, 50];

function TimerView({ p }: { p: Timer }) {
  const t = useT();
  const [custom, setCustom] = useState("");
  const parsed = parseDuration(custom);
  const submit = () => {
    if (parsed === null) return;
    p.start(parsed);
    setCustom("");
  };
  return (
    <div className="timer">
      <div className="seg">
        <button className={p.mode === "timer" ? "on" : ""} onClick={() => p.setMode("timer")}>{t("tools.timer")}</button>
        <button className={p.mode === "stopwatch" ? "on" : ""} onClick={() => p.setMode("stopwatch")}>{t("tools.stopwatch")}</button>
      </div>
      <span className="timer-time">{clock(p.ms, p.mode === "stopwatch")}</span>
      <div className="timer-main">
        <button className="primary" onClick={p.toggle}>{p.running ? t("tools.pause") : t("tools.start")}</button>
        <button onClick={p.reset}>{t("tools.reset")}</button>
      </div>
      {p.mode === "timer" && (
        <>
          <div className="presets">
            {PRESETS.map((m) => <button key={m} onClick={() => p.start(m * 60_000)}>{m}m</button>)}
          </div>
          <div className="custom">
            <input value={custom} onChange={(e) => setCustom(e.target.value)} onKeyDown={(e) => e.key === "Enter" && submit()} placeholder={t("tools.custom")} spellCheck={false} />
            <button disabled={parsed === null} onClick={submit}>{t("tools.go")}</button>
          </div>
        </>
      )}
    </div>
  );
}

export function ToolsPanel({ timer }: { timer: Timer }) {
  const t = useT();
  const [page, setPage] = useState<"tools" | "quick">("tools");
  return (
    <div className="tools-pages">
      {page === "tools" ? (
        <div className="tools">
          <Calendar />
          <TimerView p={timer} />
        </div>
      ) : (
        <QuickSettingsPage />
      )}
      <div className="page-dots">
        <button className={page === "tools" ? "on" : ""} aria-label={t("quick.page.tools")} title={t("quick.page.tools")} onClick={() => setPage("tools")} />
        <button className={page === "quick" ? "on" : ""} aria-label={t("quick.page.quick")} title={t("quick.page.quick")} onClick={() => setPage("quick")} />
      </div>
    </div>
  );
}
