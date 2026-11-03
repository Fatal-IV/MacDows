import { useEffect, useId, useRef, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import "./ui.css";

/** An on/off switch. */
export function Toggle({ checked, onChange, label }: { checked: boolean; onChange: (checked: boolean) => void; label: string }) {
  return (
    <button type="button" role="switch" aria-checked={checked} aria-label={label} className={`ui-toggle${checked ? " on" : ""}`} onClick={() => onChange(!checked)}>
      <span className="ui-toggle-knob" />
    </button>
  );
}

export interface Option {
  value: string;
  label: string;
}

/** A drop-down list. The list opens under the button and closes on a choice, Esc or a click elsewhere. */
export function Select({ value, options, onChange, label }: { value: string; options: Option[]; onChange: (value: string) => void; label: string }) {
  const [open, setOpen] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const listId = useId();
  const current = options.find((o) => o.value === value) ?? options[0];

  useEffect(() => {
    if (!open) return;
    const away = (e: PointerEvent) => !root.current?.contains(e.target as Node) && setOpen(false);
    window.addEventListener("pointerdown", away);
    return () => window.removeEventListener("pointerdown", away);
  }, [open]);

  const move = (delta: number) => {
    const at = options.findIndex((o) => o.value === value);
    const next = options[(at + delta + options.length) % options.length];
    if (next) onChange(next.value);
  };

  return (
    <div
      className="ui-select"
      ref={root}
      onKeyDown={(e) => {
        if (e.key === "Escape") setOpen(false);
        else if (e.key === "ArrowDown") {
          e.preventDefault();
          open ? move(1) : setOpen(true);
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          move(-1);
        } else if (e.key === "Enter" && open) {
          e.preventDefault();
          setOpen(false);
        }
      }}
    >
      <button type="button" className={`ui-select-button${open ? " open" : ""}`} aria-haspopup="listbox" aria-expanded={open} aria-controls={listId} aria-label={label} onClick={() => setOpen((o) => !o)}>
        <span>{current?.label}</span>
        <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden>
          <path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </button>
      <AnimatePresence>
        {open && (
          <motion.ul
            id={listId}
            role="listbox"
            className="ui-select-list"
            initial={{ opacity: 0, y: -4, scale: 0.98 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: -4, scale: 0.98 }}
            transition={{ duration: 0.12 }}
          >
            {options.map((o) => (
              <li key={o.value} role="option" aria-selected={o.value === value}>
                <button
                  type="button"
                  className={o.value === value ? "selected" : ""}
                  onClick={() => {
                    onChange(o.value);
                    setOpen(false);
                  }}
                >
                  <span>{o.label}</span>
                  {o.value === value && (
                    <svg viewBox="0 0 24 24" width="14" height="14" aria-hidden>
                      <path d="M5 12.5l4.5 4.5L19 7.5" fill="none" stroke="currentColor" strokeWidth="2.2" strokeLinecap="round" strokeLinejoin="round" />
                    </svg>
                  )}
                </button>
              </li>
            ))}
          </motion.ul>
        )}
      </AnimatePresence>
    </div>
  );
}

/** A slider with a filled track. Drag it, click the track, or use the arrow keys. */
export function Slider({ value, min, max, step, onChange, label }: { value: number; min: number; max: number; step: number; onChange: (value: number) => void; label: string }) {
  const track = useRef<HTMLDivElement>(null);
  const ratio = Math.min(1, Math.max(0, (value - min) / (max - min)));

  const setFrom = (clientX: number) => {
    const r = track.current?.getBoundingClientRect();
    if (!r || r.width === 0) return;
    const raw = min + Math.min(1, Math.max(0, (clientX - r.left) / r.width)) * (max - min);
    const stepped = Math.round(raw / step) * step;
    onChange(Number(Math.min(max, Math.max(min, stepped)).toFixed(4)));
  };

  return (
    <div
      className="ui-slider"
      role="slider"
      tabIndex={0}
      aria-label={label}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={value}
      onPointerDown={(e) => {
        e.currentTarget.setPointerCapture(e.pointerId);
        setFrom(e.clientX);
      }}
      onPointerMove={(e) => e.buttons === 1 && setFrom(e.clientX)}
      onKeyDown={(e) => {
        const delta = e.key === "ArrowRight" || e.key === "ArrowUp" ? step : e.key === "ArrowLeft" || e.key === "ArrowDown" ? -step : 0;
        if (delta) {
          e.preventDefault();
          onChange(Number(Math.min(max, Math.max(min, value + delta)).toFixed(4)));
        } else if (e.key === "Home") onChange(min);
        else if (e.key === "End") onChange(max);
      }}
    >
      <div className="ui-slider-track" ref={track}>
        <div className="ui-slider-fill" style={{ width: `${ratio * 100}%` }} />
        <div className="ui-slider-thumb" style={{ left: `${ratio * 100}%` }} />
      </div>
    </div>
  );
}

export function Button({ children, onClick, disabled, kind = "default" }: { children: ReactNode; onClick?: () => void; disabled?: boolean; kind?: "default" | "primary" }) {
  return (
    <button type="button" className={`ui-button ${kind}`} disabled={disabled} onClick={onClick}>
      {children}
    </button>
  );
}

/** A rounded card that holds rows. */
export function Group({ children }: { children: ReactNode }) {
  return <div className="ui-group">{children}</div>;
}

/** A setting: its name and explanation on the left, its control on the right. */
export function Row({ title, description, children }: { title: string; description?: string; children?: ReactNode }) {
  return (
    <div className="ui-row">
      <div className="ui-row-text">
        <div className="ui-row-title">{title}</div>
        {description && <div className="ui-row-desc">{description}</div>}
      </div>
      {children !== undefined && <div className="ui-row-control">{children}</div>}
    </div>
  );
}
