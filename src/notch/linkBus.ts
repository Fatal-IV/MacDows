// What the island shows when something connects, disconnects or fails to connect. The backend reports changes it sees
// on its own (event "link-changed"); the Control Center pages call `announceLink` for what the user just did, which
// shows even while the Control Center is open.
// "power" is the battery: `name` is the percent and the states are the charger being plugged or pulled, full and low.
export type LinkKind = "bluetooth" | "wifi" | "vpn" | "power";
export type LinkState = "connected" | "disconnected" | "failed" | "plugged" | "unplugged" | "full" | "low";

export interface Link {
  kind: LinkKind;
  name: string;
  state: LinkState;
  /** True for what the user just did on a page: it shows over whatever screen is open. */
  forced?: boolean;
}

const EVENT = "macdows-link";

export function announceLink(link: Omit<Link, "forced">) {
  window.dispatchEvent(new CustomEvent<Link>(EVENT, { detail: { ...link, forced: true } }));
}

export function onAnnouncedLink(cb: (link: Link) => void): () => void {
  const handler = (e: Event) => cb((e as CustomEvent<Link>).detail);
  window.addEventListener(EVENT, handler);
  return () => window.removeEventListener(EVENT, handler);
}

/** The same thing announced again within a few seconds is shown once. */
export function isRepeat(last: { key: string; at: number } | null, link: Link, now: number, windowMs = 8000): boolean {
  return last !== null && last.key === `${link.kind}|${link.name}|${link.state}` && now - last.at < windowMs;
}

export const linkKey = (link: Link) => `${link.kind}|${link.name}|${link.state}`;
