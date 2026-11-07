import type { IslandStyle, Visibility } from "../bridge";

const MODES: Visibility[] = ["fixed", "smart", "peek"];

/** When the island is on screen, as three tiles side by side. Each tile shows a small monitor that plays that mode in a loop (while the
 *  tile is chosen or the pointer is on it): a window slides over the island, or the pointer goes to the top edge. */
export function VisibilityPicker({ value, onChange, t, islandStyle }: { value: Visibility; onChange: (mode: Visibility) => void; t: (key: string) => string; islandStyle: IslandStyle }) {
  return (
    <div className="island-style">
      <div className="ui-row-text">
        <div className="ui-row-title">{t("settings.notch")}</div>
        <div className="ui-row-desc">{t("settings.visibilityHelp")}</div>
      </div>
      <div className="island-style-tiles three" role="radiogroup" aria-label={t("settings.notch")}>
        {MODES.map((mode) => (
          <button key={mode} type="button" role="radio" aria-checked={value === mode} className={`island-style-tile${value === mode ? " on" : ""}`} onClick={() => onChange(mode)}>
            <span className={`monitor ${islandStyle} vis-${mode}`} aria-hidden>
              <span className="monitor-screen">
                <span className="monitor-window" />
                <span className="monitor-island" />
                <span className="monitor-cursor" />
              </span>
              <span className="monitor-stand" />
            </span>
            <span className="island-style-name">{t(`vis.${mode}`)}</span>
            <span className="island-style-caption">{t(`vis.${mode}.caption`)}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
