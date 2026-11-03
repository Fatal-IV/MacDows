import type { ReactNode } from "react";

const stroke = (d: string) => <path d={d} fill="none" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" />;

export const ICONS = {
  settings: (
    <>
      <circle cx="12" cy="12" r="3" fill="none" stroke="currentColor" strokeWidth="1.7" />
      {stroke("M12 3v2.2M12 18.8V21M3 12h2.2M18.8 12H21M5.6 5.6l1.6 1.6M16.8 16.8l1.6 1.6M18.4 5.6l-1.6 1.6M7.2 16.8l-1.6 1.6")}
    </>
  ),
  explorer: stroke("M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z"),
  documents: stroke("M7 3h7l4 4v14H7V3Zm7 0v4h4M10 12h5M10 16h5"),
  downloads: stroke("M12 4v11m0 0-4-4m4 4 4-4M5 20h14"),
  pictures: (
    <>
      <rect x="3.5" y="5" width="17" height="14" rx="2" fill="none" stroke="currentColor" strokeWidth="1.7" />
      {stroke("M3.5 16l5-5 4 4 3-3 5 5")}
    </>
  ),
  power: stroke("M12 3v8M7 6.5a7.5 7.5 0 1 0 10 0"),
  calculator: (
    <>
      <rect x="5" y="3" width="14" height="18" rx="2" fill="none" stroke="currentColor" strokeWidth="1.7" />
      {stroke("M8.5 7.5h7M8.5 12h.01M12 12h.01M15.5 12h.01M8.5 16h.01M12 16h.01M15.5 16h.01")}
    </>
  ),
  chevronRight: stroke("M9 6l6 6-6 6"),
  chevronLeft: stroke("M15 6l-6 6 6 6"),
  pin: stroke("M9 4h6l-1 6 3 3H7l3-3-1-6Zm3 9v7"),
  search: (
    <>
      <circle cx="10.5" cy="10.5" r="5.5" fill="none" stroke="currentColor" strokeWidth="1.7" />
      {stroke("M15 15l5 5")}
    </>
  ),
} satisfies Record<string, ReactNode>;

export function Icon({ name, size = 18 }: { name: keyof typeof ICONS; size?: number }) {
  return (
    <svg viewBox="0 0 24 24" width={size} height={size} aria-hidden>
      {ICONS[name]}
    </svg>
  );
}
