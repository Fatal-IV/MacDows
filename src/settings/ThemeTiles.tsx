/** One theme choice (color mode, style or accent) as tiles side by side. Each tile shows a small pill drawn the way that option looks;
 *  the pill floats a little while the tile is chosen or hovered. `look` is the name of the drawing in Settings.css (`.ts-<look>`). */
export interface ThemeOption {
  value: string;
  label: string;
  look: string;
}

export function ThemeTiles({ title, value, options, onChange, columns }: { title: string; value: string; options: ThemeOption[]; onChange: (value: string) => void; columns: 2 | 3 | 4 }) {
  return (
    <div className="island-style">
      <div className={`island-style-tiles ${columns === 4 ? "four" : columns === 3 ? "three" : ""}`} role="radiogroup" aria-label={title}>
        {options.map((o) => (
          <button key={o.value} type="button" role="radio" aria-checked={value === o.value} className={`island-style-tile${value === o.value ? " on" : ""}`} onClick={() => onChange(o.value)}>
            <span className={`theme-sample ts-${o.look}`} aria-hidden>
              <i />
            </span>
            <span className="island-style-name">{o.label}</span>
          </button>
        ))}
      </div>
    </div>
  );
}
