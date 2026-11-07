import { useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { Group } from "../ui/controls";

/** A card that starts folded: its header (title and chevron) opens and closes the rows inside. */
export function FoldCard({ title, defaultOpen = false, children }: { title: string; defaultOpen?: boolean; children: ReactNode }) {
  const [open, setOpen] = useState(defaultOpen);
  const toggle = () => setOpen((v) => !v);
  return (
    <Group>
      <div className="ui-row ui-row-fold" role="button" tabIndex={0} aria-expanded={open} onClick={toggle} onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), toggle())}>
        <div className="ui-row-text">
          <div className="ui-row-title">{title}</div>
        </div>
        <div className="ui-row-control">
          <svg className={`ui-chevron${open ? " open" : ""}`} width="14" height="14" viewBox="0 0 24 24" aria-hidden>
            <path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </div>
      </div>
      <AnimatePresence initial={false}>
        {open && (
          <motion.div key="body" className="ui-fold" initial={{ height: 0, opacity: 0, overflow: "hidden" }} animate={{ height: "auto", opacity: 1, transitionEnd: { overflow: "visible" } }} exit={{ height: 0, opacity: 0, overflow: "hidden" }} transition={{ duration: 0.18 }}>
            {children}
          </motion.div>
        )}
      </AnimatePresence>
    </Group>
  );
}
