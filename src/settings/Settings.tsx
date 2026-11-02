import { setSetting, type Visibility } from "../bridge";
import { useSettings, useT } from "../i18n";
import "./Settings.css";

const VISIBILITIES: Visibility[] = ["fixed", "smart", "peek"];
const LANGUAGES = ["en", "tr"];

export default function Settings() {
  const s = useSettings();
  const t = useT();
  if (!s) return null;
  return (
    <div className="settings">
      <h1>{t("settings.title")}</h1>
      <label>
        <span>{t("settings.notch")}</span>
        <select value={s.notch_visibility} onChange={(e) => setSetting("notch_visibility", e.target.value)}>
          {VISIBILITIES.map((v) => (
            <option key={v} value={v}>{t(`vis.${v}`)}</option>
          ))}
        </select>
      </label>
      <label>
        <span>{t("settings.dock")}</span>
        <select value={s.dock_visibility} onChange={(e) => setSetting("dock_visibility", e.target.value)}>
          {VISIBILITIES.map((v) => (
            <option key={v} value={v}>{t(`vis.${v}`)}</option>
          ))}
        </select>
      </label>
      <p className="help">{t("settings.visibilityHelp")}</p>
      <label>
        <span>{t("settings.language")}</span>
        <select value={s.language} onChange={(e) => setSetting("language", e.target.value)}>
          {LANGUAGES.map((l) => (
            <option key={l} value={l}>{t(`lang.${l}`)}</option>
          ))}
        </select>
      </label>
    </div>
  );
}
