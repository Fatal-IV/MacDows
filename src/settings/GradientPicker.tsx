import { useState } from "react";
import { motion, useReducedMotion } from "framer-motion";
import { FALLBACK_PALETTE, GRADIENT_FX } from "../notch/gradientFx";

/** The gradient of the media widget as tiles: off, then the eight effects. Each tile shows its effect on a small dark island; it moves
 *  while the tile is chosen or hovered (and stands still when Windows is set to reduce motion). */
export function GradientPicker({ value, onChange, t }: { value: number; onChange: (effect: number) => void; t: (key: string) => string }) {
  const reduce = useReducedMotion();
  const [hover, setHover] = useState<number | null>(null);
  return (
    <div className="island-style">
      <div className="ui-row-text">
        <div className="ui-row-title">{t("island.music.gradient")}</div>
        <div className="ui-row-desc">{t("island.music.gradientHelp")}</div>
      </div>
      <div className="gradient-tiles" role="radiogroup" aria-label={t("island.music.gradient")}>
        {Array.from({ length: GRADIENT_FX.length + 1 }, (_, n) => {
          const fx = n > 0 ? GRADIENT_FX[n - 1] : null;
          const playing = !reduce && (value === n || hover === n);
          return (
            <button
              key={n}
              type="button"
              role="radio"
              aria-checked={value === n}
              className={`island-style-tile gradient-tile${value === n ? " on" : ""}`}
              onClick={() => onChange(n)}
              onPointerEnter={() => setHover(n)}
              onPointerLeave={() => setHover((h) => (h === n ? null : h))}
            >
              <span className="gradient-demo" aria-hidden>
                {fx && (
                  <span className="music-gradient-clip">
                    <motion.span
                      key={`${n}-${playing}`}
                      className={`music-gradient ${fx.look}`}
                      style={{ "--art": FALLBACK_PALETTE[0], "--art2": FALLBACK_PALETTE[1] } as React.CSSProperties}
                      initial={fx.from}
                      animate={playing ? fx.to : fx.still}
                      transition={playing ? { backgroundPosition: fx.move, backgroundSize: fx.move } : { duration: 0 }}
                    />
                  </span>
                )}
              </span>
              <span className="island-style-name">{t(`island.music.gradient.${n}`)}</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
