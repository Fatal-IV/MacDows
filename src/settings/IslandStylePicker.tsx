import type { IslandStyle } from "../bridge";

const STYLES: IslandStyle[] = ["notch", "dynamic"];

/** The island type as two tiles side by side. Each tile shows a small monitor with the island drawn the way that type sits on the screen. */
export function IslandStylePicker({ value, onChange, t }: { value: IslandStyle; onChange: (style: IslandStyle) => void; t: (key: string) => string }) {
  return (
    <div className="island-style">
      <div className="ui-row-text">
        <div className="ui-row-title">{t("island.style")}</div>
        <div className="ui-row-desc">{t("island.styleHelp")}</div>
      </div>
      <div className="island-style-tiles" role="radiogroup" aria-label={t("island.style")}>
        {STYLES.map((style) => (
          <button key={style} type="button" role="radio" aria-checked={value === style} className={`island-style-tile${value === style ? " on" : ""}`} onClick={() => onChange(style)}>
            <span className={`monitor ${style}`} aria-hidden>
              <span className="monitor-screen">
                <span className="monitor-island" />
              </span>
              <span className="monitor-stand" />
            </span>
            <span className="island-style-name">{t(`island.style.${style}`)}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
