import { useEffect, useRef, useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import "./radial.css";

export interface RadialItem {
  id: string;
  label: string;
  icon: ReactNode;
}

const ITEM_WIDTH = 104;
const CIRCLE = 66;
const CENTER = 92;

/** A button in the middle; when it is pressed the items fan out in a circle around it. */
export default function RadialMenu({
  items,
  open,
  onToggle,
  onPick,
  toggleLabel,
  hint,
}: {
  items: RadialItem[];
  open: boolean;
  onToggle: () => void;
  onPick: (id: string) => void;
  toggleLabel: string;
  hint: string;
}) {
  const stage = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 760, h: 560 });

  useEffect(() => {
    const el = stage.current;
    if (!el) return;
    const measure = () => setSize({ w: el.clientWidth, h: el.clientHeight });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // The ring has to fit in the window, whatever its size.
  const radius = Math.max(110, Math.min(200, size.w / 2 - ITEM_WIDTH / 2 - 24, size.h * 0.46 - CIRCLE - 24));
  const cy = size.h * 0.46;

  return (
    <div className="radial" ref={stage}>
      <motion.div
        className="radial-guide"
        style={{ top: cy, width: radius * 2, height: radius * 2 }}
        initial={false}
        animate={{ opacity: open ? 1 : 0, scale: open ? 1 : 0.4 }}
        transition={{ type: "spring", stiffness: 260, damping: 26 }}
      />
      {items.map((item, i) => {
        const angle = (-90 + (360 / items.length) * i) * (Math.PI / 180);
        return (
          <motion.button
            key={item.id}
            type="button"
            className="radial-item"
            style={{ top: cy - CIRCLE / 2, width: ITEM_WIDTH, marginLeft: -ITEM_WIDTH / 2, pointerEvents: open ? "auto" : "none" }}
            tabIndex={open ? 0 : -1}
            aria-hidden={!open}
            initial={false}
            animate={{
              x: open ? Math.cos(angle) * radius : 0,
              y: open ? Math.sin(angle) * radius : 0,
              scale: open ? 1 : 0.3,
              opacity: open ? 1 : 0,
            }}
            transition={{ type: "spring", stiffness: 300, damping: 24, delay: open ? i * 0.035 : 0 }}
            whileHover={{ scale: open ? 1.08 : 0.3 }}
            whileTap={{ scale: open ? 0.96 : 0.3 }}
            onClick={() => open && onPick(item.id)}
          >
            <span className="radial-circle">
              <svg viewBox="0 0 24 24" width="26" height="26" aria-hidden>
                {item.icon}
              </svg>
            </span>
            <span className="radial-label">{item.label}</span>
          </motion.button>
        );
      })}

      <motion.button
        type="button"
        className="radial-center"
        style={{ top: cy - CENTER / 2, width: CENTER, height: CENTER, marginLeft: -CENTER / 2 }}
        aria-expanded={open}
        aria-label={toggleLabel}
        onClick={onToggle}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.95 }}
      >
        <motion.svg viewBox="0 0 24 24" width="34" height="34" aria-hidden animate={{ rotate: open ? 135 : 0 }} transition={{ type: "spring", stiffness: 260, damping: 20 }}>
          <path d="M12 5v14M5 12h14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </motion.svg>
      </motion.button>
      <motion.p className="radial-hint" style={{ top: cy + CENTER / 2 + 14 }} initial={false} animate={{ opacity: open ? 0 : 1 }}>
        {hint}
      </motion.p>
    </div>
  );
}
