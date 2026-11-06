import type { Link } from "./linkBus";

/** Something connected, disconnected or failed to connect: an animated icon (green, amber or red) and the name, inside the island. */
export function LinkToast({ link, label }: { link: Link; label: string }) {
  return (
    <div className={`link-toast ${link.kind} ${link.state}`}>
      <span className="link-icon" aria-hidden>
        <i className="link-ring" />
        <i className="link-ring late" />
        <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          {link.kind === "bluetooth" && <path className="bt-rune" pathLength={1} d="M6.5 7.5 17 17l-5 5V2l5 5L6.5 16.5" />}
          {link.kind === "wifi" && (
            <>
              <path className="wf-arc a3" d="M2.5 9a14 14 0 0 1 19 0" />
              <path className="wf-arc a2" d="M5.5 12.6a9.5 9.5 0 0 1 13 0" />
              <path className="wf-arc a1" d="M8.6 16a5 5 0 0 1 6.8 0" />
              <circle className="wf-dot" cx="12" cy="19.4" r="1.3" fill="currentColor" stroke="none" />
            </>
          )}
          {link.kind === "vpn" && (
            <>
              <path className="vpn-shield" pathLength={1} d="M12 2.8 4.6 5.8v5.6c0 4.5 3 8 7.4 9.4 4.4-1.4 7.4-4.9 7.4-9.4V5.8L12 2.8Z" />
              <path className="vpn-check" pathLength={1} d="m8.8 12 2.2 2.2 4.2-4.4" />
            </>
          )}
          {link.state !== "connected" && <path className="link-slash" pathLength={1} d="M3.5 3.5 20.5 20.5" />}
        </svg>
      </span>
      <span className="link-text">
        <span className="link-label">{label}</span>
        <span className="link-name">{link.name}</span>
      </span>
    </div>
  );
}
