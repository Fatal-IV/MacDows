import { useEffect, useRef, useState, type ReactNode } from "react";
import { motion } from "framer-motion";
import "./radial.css";

export interface RadialItem {
  id: string;
  label: string;
  icon: ReactNode;
}

const ITEM_WIDTH = 90;
const CIRCLE = 62;
const CENTER = 60;
const MARGIN_BOTTOM = 22;

/**
 * A button at the bottom center of the window. When it is pressed the items fan out around it in an arc, and the
 * rest of the window is dimmed; picking an item, pressing the button again or clicking outside closes it.
 */
export default function RadialMenu({
  items,
  activeId,
  open,
  onToggle,
  onPick,
  toggleLabel,
}: {
  items: RadialItem[];
  activeId: string;
  open: boolean;
  onToggle: () => void;
  onPick: (id: string) => void;
  toggleLabel: string;
}) {
  const stage = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 860, h: 620 });

  useEffect(() => {
    const el = stage.current;
    if (!el) return;
    const measure = () => setSize({ w: el.clientWidth, h: el.clientHeight });
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  // A tight arc close above the button, wider than tall like a rainbow; it still has to fit a small window.
  const radiusX = Math.max(120, Math.min(205, size.w / 2 - ITEM_WIDTH / 2 - 16));
  const radiusY = Math.max(90, Math.min(125, size.h - CENTER - MARGIN_BOTTOM - CIRCLE - 56));
  const cy = size.h - MARGIN_BOTTOM - CENTER / 2;

  return (
    <div className="radial" ref={stage}>
      {/* Invisible: a click anywhere outside the buttons closes the menu. The page is not dimmed or blurred. */}
      <div className="radial-scrim" style={{ pointerEvents: open ? "auto" : "none" }} onClick={onToggle} />
      {items.map((item, i) => {
        // From just above the left edge to just above the right edge.
        const angle = (180 + (180 / (items.length + 1)) * (i + 1)) * (Math.PI / 180);
        const active = item.id === activeId;
        return (
          <motion.button
            key={item.id}
            type="button"
            className={`radial-item${active ? " active" : ""}`}
            style={{ top: cy - CIRCLE / 2, width: ITEM_WIDTH, marginLeft: -ITEM_WIDTH / 2, pointerEvents: open ? "auto" : "none" }}
            tabIndex={open ? 0 : -1}
            aria-hidden={!open}
            aria-current={active ? "page" : undefined}
            initial={false}
            animate={{
              x: open ? Math.cos(angle) * radiusX : 0,
              y: open ? Math.sin(angle) * radiusY : 0,
              scale: open ? 1 : 0.3,
              opacity: open ? 1 : 0,
            }}
            transition={{ type: "spring", stiffness: 300, damping: 24, delay: open ? i * 0.035 : 0 }}
            whileHover={{ scale: open ? 1.08 : 0.3 }}
            whileTap={{ scale: open ? 0.96 : 0.3 }}
            onClick={() => open && onPick(item.id)}
          >
            <span className="radial-circle">
              <svg viewBox="0 0 24 24" width="24" height="24" aria-hidden>
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
        title={toggleLabel}
        onClick={onToggle}
        whileHover={{ scale: 1.06 }}
        whileTap={{ scale: 0.94 }}
      >
        <motion.svg viewBox="0 0 24 24" width="26" height="26" aria-hidden animate={{ rotate: open ? 135 : 0 }} transition={{ type: "spring", stiffness: 260, damping: 20 }}>
          <path d="M12 5v14M5 12h14" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
        </motion.svg>
      </motion.button>
    </div>
  );
}
