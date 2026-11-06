export interface Link {
  kind: "bluetooth" | "wifi";
  name: string;
}

/** A device or network just connected: an animated icon and its name, inside the island. */
export function LinkToast({ link, label }: { link: Link; label: string }) {
  return (
    <div className={`link-toast ${link.kind}`}>
      <span className="link-icon" aria-hidden>
        <i className="link-ring" />
        <i className="link-ring late" />
        {link.kind === "bluetooth" ? (
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path className="bt-rune" pathLength={1} d="M6.5 7.5 17 17l-5 5V2l5 5L6.5 16.5" />
          </svg>
        ) : (
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path className="wf-arc a3" d="M2.5 9a14 14 0 0 1 19 0" />
            <path className="wf-arc a2" d="M5.5 12.6a9.5 9.5 0 0 1 13 0" />
            <path className="wf-arc a1" d="M8.6 16a5 5 0 0 1 6.8 0" />
            <circle className="wf-dot" cx="12" cy="19.4" r="1.3" fill="currentColor" stroke="none" />
          </svg>
        )}
      </span>
      <span className="link-text">
        <span className="link-label">{label}</span>
        <span className="link-name">{link.name}</span>
      </span>
    </div>
  );
}
