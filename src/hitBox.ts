// The box a bar's window is cut down to (CSS pixels, relative to the window). Pure helpers; the hook that sends it
// to the backend lives in hitRegion.ts.

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** The smallest box holding both. */
export function union(a: Box, b: Box): Box {
  const left = Math.min(a.x, b.x);
  const top = Math.min(a.y, b.y);
  const right = Math.max(a.x + a.w, b.x + b.w);
  const bottom = Math.max(a.y + a.h, b.y + b.h);
  return { x: left, y: top, w: right - left, h: bottom - top };
}

/** `box` grown by the given margins (a margin can be 0). */
export function pad(box: Box, m: { left: number; right: number; top: number; bottom: number }): Box {
  return { x: box.x - m.left, y: box.y - m.top, w: box.w + m.left + m.right, h: box.h + m.top + m.bottom };
}

/** What to send when the shape changes: while it grows or moves, the old and the new shape together (the spring
 *  animation is still on its way); the first time, just the new one. */
export function transitionBox(sent: Box | null, next: Box): Box {
  return sent ? union(sent, next) : next;
}

export const sameBox = (a: Box | null, b: Box | null): boolean => (a === null || b === null ? a === b : a.x === b.x && a.y === b.y && a.w === b.w && a.h === b.h);

/** The island's shape in its window: centered at the top, `width` x `height` (already multiplied by the UI scale),
 *  with room around it for its shadow and the spring's overshoot. */
export function islandBox(windowWidth: number, width: number, height: number, margin: number): Box {
  return pad({ x: (windowWidth - width) / 2, y: 0, w: width, h: height }, { left: margin, right: margin, top: 0, bottom: margin });
}

/** The dock's shape: its own box with side margins, room above for hover pop-ups, down to the bottom of the window. */
export function dockBox(dock: Box, windowHeight: number, side: number, above: number): Box {
  const top = Math.max(0, dock.y - above);
  return { x: dock.x - side, y: top, w: dock.w + 2 * side, h: windowHeight - top };
}
