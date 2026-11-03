import { useState } from "react";
import { STYLE_OPACITY, THEME_STYLES } from "../appearance/theme";
import { useT } from "../i18n";

const MODES = ["dark", "light", "auto", "adaptive"] as const;
const ACCENTS = ["mono", "system"] as const;

/** A localStorage-backed value. Other windows pick the change up through the `storage` event. */
function useStored(key: string, fallback: string): [string, (v: string) => void] {
  const [value, setValue] = useState(() => localStorage.getItem(key) ?? fallback);
  return [
    value,
    (v) => {
      localStorage.setItem(key, v);
      setValue(v);
    },
  ];
}

export default function Appearance() {
  const t = useT();
  const [mode, setMode] = useStored("nectar-theme-mode", "dark");
  const [style, setStyle] = useStored("nectar-theme-style", "solid");
  const [accent, setAccent] = useStored("nectar-theme-accent", "mono");
  const [opacity, setOpacity] = useStored("nectar-theme-opacity", "0.8");
  const [scale, setScale] = useStored("nectar-scale", "1");

  return (
    <section className="appearance">
      <h2>{t("appearance.title")}</h2>
      <label>
        <span>{t("appearance.mode")}</span>
        <select value={mode} onChange={(e) => setMode(e.target.value)}>
          {MODES.map((m) => (
            <option key={m} value={m}>{t(`appearance.mode.${m}`)}</option>
          ))}
        </select>
      </label>
      <label>
        <span>{t("appearance.style")}</span>
        <select
          value={style}
          onChange={(e) => {
            setStyle(e.target.value);
            // Each style is designed around its own opacity; the slider can still change it.
            setOpacity(String(STYLE_OPACITY[e.target.value] ?? 0.8));
          }}
        >
          {THEME_STYLES.map((s) => (
            <option key={s} value={s}>{t(`appearance.style.${s}`)}</option>
          ))}
        </select>
      </label>
      <label>
        <span>{t("appearance.accent")}</span>
        <select value={accent} onChange={(e) => setAccent(e.target.value)}>
          {ACCENTS.map((a) => (
            <option key={a} value={a}>{t(`appearance.accent.${a}`)}</option>
          ))}
        </select>
      </label>
      <label>
        <span>{t("appearance.opacity")}: {Math.round(parseFloat(opacity) * 100)}%</span>
        <input type="range" min="0.3" max="1" step="0.01" value={opacity} onChange={(e) => setOpacity(e.target.value)} />
      </label>
      <label>
        <span>{t("appearance.scale")}: {Math.round(parseFloat(scale) * 100)}%</span>
        <input type="range" min="0.8" max="1.2" step="0.05" value={scale} onChange={(e) => setScale(e.target.value)} />
      </label>
    </section>
  );
}
