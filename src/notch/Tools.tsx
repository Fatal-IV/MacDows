// The calendar and the Pomodoro timer that the notch shows besides the media panel.
import { useEffect, useMemo, useState } from "react";
import { useT } from "../i18n";

const FOCUS_SECONDS = 25 * 60;
const BREAK_SECONDS = 5 * 60;

export interface Pomodoro {
  phase: "focus" | "break";
  remaining: number;
  running: boolean;
  toggle: () => void;
  reset: () => void;
  skip: () => void;
}

const length = (phase: "focus" | "break") => (phase === "focus" ? FOCUS_SECONDS : BREAK_SECONDS);

/** Lives in the notch itself so the timer keeps running while the panel is closed. */
export function usePomodoro(): Pomodoro {
  const [phase, setPhase] = useState<"focus" | "break">("focus");
  const [remaining, setRemaining] = useState(FOCUS_SECONDS);
  const [endAt, setEndAt] = useState<number | null>(null);

  useEffect(() => {
    if (endAt === null) return;
    const id = setInterval(() => {
      const left = Math.max(0, Math.round((endAt - Date.now()) / 1000));
      setRemaining(left);
      if (left === 0) {
        // The phase ends: switch to the other one and stop, waiting for the user.
        const next = phase === "focus" ? "break" : "focus";
        setPhase(next);
        setRemaining(length(next));
        setEndAt(null);
      }
    }, 250);
    return () => clearInterval(id);
  }, [endAt, phase]);

  return {
    phase,
    remaining,
    running: endAt !== null,
    toggle: () => setEndAt((e) => (e === null ? Date.now() + remaining * 1000 : null)),
    reset: () => {
      setEndAt(null);
      setRemaining(length(phase));
    },
    skip: () => {
      const next = phase === "focus" ? "break" : "focus";
      setEndAt(null);
      setPhase(next);
      setRemaining(length(next));
    },
  };
}

function Calendar() {
  const [offset, setOffset] = useState(0);
  const today = new Date();
  const first = new Date(today.getFullYear(), today.getMonth() + offset, 1);
  const lang = document.documentElement.lang || undefined;
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
        <button onClick={() => setOffset(offset - 1)} aria-label="‹">‹</button>
        <button className="cal-title" onClick={() => setOffset(0)}>{title}</button>
        <button onClick={() => setOffset(offset + 1)} aria-label="›">›</button>
      </div>
      <div className="cal-grid">
        {names.map((n, i) => <span key={`n${i}`} className="cal-name">{n}</span>)}
        {cells.map((d, i) => <span key={i} className={`cal-day${d && isToday(d) ? " today" : ""}`}>{d ?? ""}</span>)}
      </div>
    </div>
  );
}

const mmss = (s: number) => `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;

function PomodoroView({ p }: { p: Pomodoro }) {
  const t = useT();
  const ratio = 1 - p.remaining / length(p.phase);
  return (
    <div className="pomodoro">
      <span className="pomo-phase">{p.phase === "focus" ? t("tools.focus") : t("tools.break")}</span>
      <span className="pomo-time">{mmss(p.remaining)}</span>
      <div className="bar-track"><div className="bar-fill" style={{ width: `${ratio * 100}%` }} /></div>
      <div className="pomo-buttons">
        <button onClick={p.toggle}>{p.running ? t("tools.pause") : t("tools.start")}</button>
        <button onClick={p.reset}>{t("tools.reset")}</button>
        <button onClick={p.skip}>{t("tools.skip")}</button>
      </div>
    </div>
  );
}

export function ToolsPanel({ pomodoro }: { pomodoro: Pomodoro }) {
  return (
    <div className="tools">
      <Calendar />
      <PomodoroView p={pomodoro} />
    </div>
  );
}
