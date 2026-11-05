// Status widget icons: stroke drawings on a 24x24 grid, drawn for MacDows (no SF Symbols).
import type { WeatherIcon } from "./widgetLogic";

const stroke = { fill: "none", stroke: "currentColor", strokeWidth: 1.8, strokeLinecap: "round", strokeLinejoin: "round" } as const;
const CLOUD = "M7 18a4 4 0 0 1-.4-7.98A5.5 5.5 0 0 1 17.3 9.2 4.4 4.4 0 0 1 17 18Z";
const SMALL_CLOUD = "M7 15a4 4 0 0 1-.4-7.98A5.5 5.5 0 0 1 17.3 6.2 4.4 4.4 0 0 1 17 15Z";

export function WeatherGlyph({ icon, size = 14 }: { icon: WeatherIcon; size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" {...stroke} aria-hidden>
      {icon === "clear" && (
        <>
          <circle cx="12" cy="12" r="4" />
          <path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M5.3 18.7l1.6-1.6M17.1 6.9l1.6-1.6" />
        </>
      )}
      {icon === "partly" && (
        <>
          <path d="M8.5 5.5A3.5 3.5 0 0 1 14 8M4 8.5h1M7 3.5v1M3.6 5.3l.8.8" />
          <path d="M8 19a3.6 3.6 0 0 1-.3-7.2 5 5 0 0 1 9.6.9A3.8 3.8 0 0 1 17 19Z" />
        </>
      )}
      {icon === "cloudy" && <path d={CLOUD} />}
      {icon === "fog" && <path d="M4 8h16M3 12h18M5 16h14M8 20h8" />}
      {icon === "rain" && (
        <>
          <path d={SMALL_CLOUD} />
          <path d="M8 18l-1 3M12 18l-1 3M16 18l-1 3" />
        </>
      )}
      {icon === "snow" && (
        <>
          <path d={SMALL_CLOUD} />
          <path d="M8 18.5v.01M12 20v.01M16 18.5v.01" strokeWidth={2.6} />
        </>
      )}
      {icon === "storm" && (
        <>
          <path d={SMALL_CLOUD} />
          <path d="M12.5 13l-2.5 4h3l-1.5 4" />
        </>
      )}
    </svg>
  );
}

export type StatGlyphId = "battery" | "cpu" | "ram" | "disk" | "net";

export function StatGlyph({ id, size = 14 }: { id: StatGlyphId; size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" {...stroke} aria-hidden>
      {id === "battery" && (
        <>
          <rect x="2.5" y="7.5" width="16" height="9" rx="2.2" />
          <path d="M21 10.5v3" />
        </>
      )}
      {id === "cpu" && (
        <>
          <rect x="6" y="6" width="12" height="12" rx="2" />
          <rect x="9.5" y="9.5" width="5" height="5" rx="0.8" />
          <path d="M9 2.8v3.2M15 2.8v3.2M9 18v3.2M15 18v3.2M2.8 9h3.2M2.8 15h3.2M18 9h3.2M18 15h3.2" />
        </>
      )}
      {id === "ram" && (
        <>
          <rect x="2.5" y="7" width="19" height="9" rx="1.6" />
          <path d="M6 16v2.5M10 16v2.5M14 16v2.5M18 16v2.5M6.5 10.5v2M10.5 10.5v2M14.5 10.5v2" />
        </>
      )}
      {id === "disk" && (
        <>
          <ellipse cx="12" cy="6.5" rx="7.5" ry="3" />
          <path d="M4.5 6.5v11c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3v-11M4.5 12c0 1.7 3.4 3 7.5 3s7.5-1.3 7.5-3" />
        </>
      )}
      {id === "net" && <path d="M12 4v13M6.5 11.5 12 17l5.5-5.5M5 21h14" />}
    </svg>
  );
}
