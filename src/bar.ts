/** Which bar a window label belongs to: the island ("notch", "notch-2", ...), the dock ("dock", "dock-2", ...), or none of them.
 *  Mirrors `bar.rs`. */
export function barKind(label: string): "notch" | "dock" | null {
  const m = /^(notch|dock)(-\d+)?$/.exec(label);
  return m ? (m[1] as "notch" | "dock") : null;
}
