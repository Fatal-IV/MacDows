import type { IslandStyle, Visibility } from "../bridge";

const MODES: Visibility[] = ["fixed", "smart", "peek"];

/** When the island (or the dock, `edge="bottom"`) is on screen, as three tiles side by side. Each tile shows a small monitor that plays that mode in a loop (while the
 *  tile is chosen or the pointer is on it): a window slides over the island, or the pointer goes to the top edge. */
export function VisibilityPicker({ value, onChange, t, islandStyle, edge = "top" }: { value: Visibility; onChange: (mode: Visibility) => void; t: (key: string) => string; islandStyle?: IslandStyle; edge?: "top" | "bottom" }) {
  // The dock sits at the bottom edge (a pill); the island at the top, as its type says.
  const title = edge === "bottom" ? t("settings.dock") : t("settings.notch");
  return (
    <div className="island-style">
      <div className="ui-row-text">
        <div className="ui-row-title">{title}</div>
        <div className="ui-row-desc">{t("settings.visibilityHelp")}</div>
      </div>
      <div className="island-style-tiles three" role="radiogroup" aria-label={title}>
        {MODES.map((mode) => (
          <button key={mode} type="button" role="radio" aria-checked={value === mode} className={`island-style-tile${value === mode ? " on" : ""}`} onClick={() => onChange(mode)}>
            <span className={`monitor ${edge === "bottom" ? "edge-bottom" : islandStyle ?? "notch"} vis-${mode}`} aria-hidden>
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
