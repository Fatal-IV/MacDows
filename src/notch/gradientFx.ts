// The moving gradients of the media widget: shared by the island (which draws them) and Settings (which shows each one on its tile).

/**
 * The moving gradients behind the player and the small island (`look` is the pattern in DynamicIsland.css, `from`/`to`/`move` is how
 * framer-motion slides it, `still` is the resting picture for people who asked for less motion). The effect is the one picked in Settings (1 to 8 there; 0 is off).
 * Directional ones slide a seamless 1-D pattern by exactly one period; the diagonals slide the same pattern on a larger turned layer,
 * because a tiled 135 degree gradient would show a seam. Moving the pattern by p% of (box - image) is one period when
 * p = size / (1 - size): -200% / +200% for the 200% patterns, +100% for the 50% pattern of the diagonals.
 */
export const GRADIENT_FX = [
  // 1: left to right
  { look: "fx0", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "-200% 0%" }, still: { backgroundPosition: "0% 0%" }, move: { duration: 6, ease: "linear", repeat: Infinity } },
  // 2: right to left
  { look: "fx0", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "200% 0%" }, still: { backgroundPosition: "0% 0%" }, move: { duration: 6, ease: "linear", repeat: Infinity } },
  // 3: top to bottom
  { look: "fx1", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "0% -200%" }, still: { backgroundPosition: "0% 0%" }, move: { duration: 7, ease: "linear", repeat: Infinity } },
  // 4: bottom to top
  { look: "fx1", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "0% 200%" }, still: { backgroundPosition: "0% 0%" }, move: { duration: 7, ease: "linear", repeat: Infinity } },
  // 5: diagonal, top left to bottom right
  { look: "diag diag1", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "100% 0%" }, still: { backgroundPosition: "0% 0%" }, move: { duration: 5, ease: "linear", repeat: Infinity } },
  // 6: diagonal, bottom left to top right
  { look: "diag diag2", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "100% 0%" }, still: { backgroundPosition: "0% 0%" }, move: { duration: 5, ease: "linear", repeat: Infinity } },
  // 7: aurora: two soft spots of color drift around
  { look: "fx2", from: { backgroundPosition: "0% 0%" }, to: { backgroundPosition: "100% 100%" }, still: { backgroundPosition: "50% 50%" }, move: { duration: 7, ease: "easeInOut", repeat: Infinity, repeatType: "reverse" as const } },
  // 8: a light that breathes from the middle
  { look: "fx3", from: { backgroundSize: "90% 90%" }, to: { backgroundSize: "240% 240%" }, still: { backgroundSize: "150% 150%" }, move: { duration: 4, ease: "easeInOut", repeat: Infinity, repeatType: "reverse" as const } },
] as const;

/** Used while a source gives no cover (a video, a Short): a soft blue and violet. */
export const FALLBACK_PALETTE: [string, string] = ["110, 125, 200", "150, 110, 190"];
