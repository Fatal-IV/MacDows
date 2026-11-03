// Simple transport icons drawn for MacDows (Nectar uses SF Symbols glyphs, which are not
// redistributable outside Apple platforms, so they are intentionally not copied).

interface IconProps {
  size?: number;
  className?: string;
}

export function PlayIcon({ size = 14, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className={className} style={{ marginLeft: 1 }} aria-hidden>
      <path d="M7 4.6v14.8a1 1 0 0 0 1.5.86l12-7.4a1 1 0 0 0 0-1.72l-12-7.4A1 1 0 0 0 7 4.6Z" />
    </svg>
  );
}

export function PauseIcon({ size = 14, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className={className} aria-hidden>
      <rect x="6" y="4" width="4.2" height="16" rx="1.2" />
      <rect x="13.8" y="4" width="4.2" height="16" rx="1.2" />
    </svg>
  );
}

export function SkipBackIcon({ size = 28, className }: IconProps) {
  return (
    <svg width={size} height={size / 2} viewBox="0 0 32 16" fill="currentColor" className={className} aria-hidden>
      <path d="M15 2.6v10.8a.9.9 0 0 1-1.4.74L7 8.74v4.66a1 1 0 0 1-2 0V2.6a1 1 0 0 1 2 0v4.66l6.6-5.4A.9.9 0 0 1 15 2.6Z" transform="translate(2 0)" />
      <path d="M26 2.6v10.8a.9.9 0 0 1-1.4.74L16 8.74V7.26l8.6-5.4A.9.9 0 0 1 26 2.6Z" transform="translate(1 0)" />
    </svg>
  );
}

export function SkipForwardIcon({ size = 28, className }: IconProps) {
  return (
    <svg width={size} height={size / 2} viewBox="0 0 32 16" fill="currentColor" className={className} style={{ transform: "scaleX(-1)" }} aria-hidden>
      <path d="M15 2.6v10.8a.9.9 0 0 1-1.4.74L7 8.74v4.66a1 1 0 0 1-2 0V2.6a1 1 0 0 1 2 0v4.66l6.6-5.4A.9.9 0 0 1 15 2.6Z" transform="translate(2 0)" />
      <path d="M26 2.6v10.8a.9.9 0 0 1-1.4.74L16 8.74V7.26l8.6-5.4A.9.9 0 0 1 26 2.6Z" transform="translate(1 0)" />
    </svg>
  );
}

export function MusicNoteIcon({ size = 32, className }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="currentColor" className={className} aria-hidden>
      <path d="M9 4.5a1 1 0 0 1 .8-.98l9-1.8A1 1 0 0 1 20 2.7V16a3.5 3.5 0 1 1-2-3.16V6.3l-7 1.4V18a3.5 3.5 0 1 1-2-3.16V4.5Z" />
    </svg>
  );
}
