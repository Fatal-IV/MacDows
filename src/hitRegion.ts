import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { clearHitRegion, setHitRegion } from "./bridge";

/** The label of this bar window ("notch", "dock", "dock-2", ...). */
const label = getCurrentWindow().label;
import { sameBox, transitionBox, type Box } from "./hitBox";

/** How long the island or dock may need to reach its new size; until then the old and new shape are both kept. */
const SETTLE_MS = 600;

/** Cuts the bar's window down to `box` (CSS pixels in the window), so the empty rest of the window no longer takes
 *  clicks. A null box gives the whole window back. */
export function useHitRegion(box: Box | null, radius: number) {
  const sent = useRef<Box | null>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const x = box?.x, y = box?.y, w = box?.w, h = box?.h;

  useEffect(() => {
    clearTimeout(timer.current);
    if (x === undefined || y === undefined || w === undefined || h === undefined) {
      sent.current = null;
      clearHitRegion(label).catch(() => {});
      return;
    }
    const next: Box = { x, y, w, h };
    const send = (b: Box) => {
      if (sameBox(sent.current, b)) return;
      sent.current = b;
      const d = window.devicePixelRatio || 1;
      setHitRegion(label, Math.round(b.x * d), Math.round(b.y * d), Math.round(b.w * d), Math.round(b.h * d), Math.round(radius * d)).catch(() => {});
    };
    send(transitionBox(sent.current, next));
    timer.current = setTimeout(() => send(next), SETTLE_MS);
  }, [x, y, w, h, radius]);

  useEffect(() => () => clearTimeout(timer.current), []);
}
