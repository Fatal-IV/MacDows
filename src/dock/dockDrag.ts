/** How far (px) a pin may be dropped above or below the dock before the drag counts as cancelled. */
const DROP_MARGIN = 60;

/** Was the pointer let go well away from the dock? Then the drag is cancelled and the old order stays. */
export function droppedOutside(pointerY: number, dockTop: number, dockBottom: number): boolean {
  return pointerY < dockTop - DROP_MARGIN || pointerY > dockBottom + DROP_MARGIN;
}

export function sameOrder(a: string[], b: string[]): boolean {
  return a.length === b.length && a.every((id, i) => id === b[i]);
}
