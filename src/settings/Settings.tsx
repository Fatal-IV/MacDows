import { setSetting, type Visibility } from "../bridge";
import { useSettings, useT } from "../i18n";
import { checkForUpdate, installUpdate, useAppVersion, useUpdateStatus, type UpdateStatus } from "../updates";
import "./Settings.css";

const VISIBILITIES: Visibility[] = ["fixed", "smart", "peek"];
const LANGUAGES = ["en", "tr"];

function statusLine(t: (key: string) => string, status: UpdateStatus): string {
  switch (status.kind) {
    case "idle":
      return t("update.idle");
    case "checking":
      return t("update.checking");
    case "up_to_date":
      return t("update.upToDate");
    case "available":
      return t("update.available").replace("{v}", status.version);
    case "installing":
      return status.percent == null ? t("update.installing") : `${t("update.installing")} ${status.percent}%`;
    case "error":
      return t(`update.error.${status.message}`);
  }
}

export default function Settings() {
  const s = useSettings();
  const t = useT();
  const update = useUpdateStatus();
  const version = useAppVersion();
  if (!s) return null;
  const busy = update.kind === "checking" || update.kind === "installing";
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

      <label className="check">
        <input type="checkbox" checked={s.replace_taskbar} onChange={(e) => setSetting("replace_taskbar", e.target.checked)} />
        <span>{t("settings.replaceTaskbar")}</span>
      </label>
      <p className="help">{t("settings.replaceTaskbarHelp")}</p>
      <label className="check">
        <input type="checkbox" checked={s.start_with_windows} onChange={(e) => setSetting("start_with_windows", e.target.checked)} />
        <span>{t("settings.startWithWindows")}</span>
      </label>

      <section className="updates">
        <h2>{t("update.title")}</h2>
        <div className="row">
          <span>{t("update.version")}</span>
          <strong>{version}</strong>
        </div>
        <p className={update.kind === "error" ? "status bad" : "status"}>{statusLine(t, update)}</p>
        {update.kind === "available" && update.notes && (
          <div className="notes">
            <div className="notes-title">{t("update.notes")}</div>
            <pre>{update.notes}</pre>
          </div>
        )}
        <div className="buttons">
          <button disabled={busy} onClick={() => checkForUpdate().catch(() => {})}>
            {t("update.checkNow")}
          </button>
          {update.kind === "available" && (
            <button className="primary" disabled={busy} onClick={() => installUpdate().catch(() => {})}>
              {t("update.install")}
            </button>
          )}
        </div>
        <label className="check">
          <input type="checkbox" checked={s.check_updates} onChange={(e) => setSetting("check_updates", e.target.checked)} />
          <span>{t("update.auto")}</span>
        </label>
      </section>
    </div>
  );
}
