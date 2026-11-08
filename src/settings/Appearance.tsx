import { useEffect, useState } from "react";
import { THEME_STYLES } from "../appearance/theme";
import { useT } from "../i18n";
import { Row, Select, Toggle } from "../ui/controls";
import { isLightTheme } from "../appearance/lightMode";
import { FoldCard } from "./FoldCard";
import { ThemeTiles } from "./ThemeTiles";
import { useStored } from "./useStored";

const MODES = ["auto", "light", "dark"] as const;
// The Start menu tint is on by default and has no switch for now; set this to show it again.
const SHOW_START_TINT = false;
const LIGHT_CONTRASTS = ["none", "outline", "gray"] as const;

/** Whether the theme is light right now ("follow Windows" is light only while Windows is light). */
function useLightTheme(): boolean {
  const [mode] = useStored("nectar-theme-mode", "auto");
  const [systemDark, setSystemDark] = useState(() => !!window.matchMedia?.("(prefers-color-scheme: dark)").matches);
  useEffect(() => {
    const query = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!query) return;
    const onChange = () => setSystemDark(query.matches);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);
  return isLightTheme(mode, systemDark);
}

/** How the island stands out on a light theme (white windows behind a dark island). Island only; shown only on a light theme. */
export function LightContrastRow() {
  const t = useT();
  const light = useLightTheme();
  const [lightVariant, setLightVariant] = useStored("macdows-light-variant", "none");
  if (!light) return null;
  return (
    <Row title={t("appearance.lightContrast")} description={t("appearance.lightContrastHelp")}>
      <Select label={t("appearance.lightContrast")} value={lightVariant} onChange={setLightVariant} options={LIGHT_CONTRASTS.map((c) => ({ value: c, label: t(`appearance.lightContrast.${c}`) }))} />
    </Row>
  );
}

/** The theme shared by the island and the dock. */
export default function Appearance() {
  const t = useT();
  const [mode, setMode] = useStored("nectar-theme-mode", "auto");
  const [storedStyle, setStyle] = useStored("nectar-theme-style", "solid");
  // A style that no longer exists (the old glass styles) shows as Solid, which is what the theme falls back to.
  const style = (THEME_STYLES as readonly string[]).includes(storedStyle) ? storedStyle : "solid";
  const [startTint, setStartTint] = useStored("macdows-start-tint", "true");

  return (
    <>
      <FoldCard title={t("appearance.mode")}>
        <ThemeTiles title={t("appearance.mode")} value={mode} onChange={setMode} columns={3} options={MODES.map((m) => ({ value: m, label: t(`appearance.mode.${m}`), look: m }))} />
      </FoldCard>
      <FoldCard title={t("appearance.style")}>
        <ThemeTiles
          title={t("appearance.style")}
          value={style}
          columns={2}
          onChange={setStyle}
          options={THEME_STYLES.map((x) => ({ value: x, label: t(`appearance.style.${x}`), look: x }))}
        />
      </FoldCard>
      {SHOW_START_TINT && (
        <FoldCard title={t("appearance.startMenu")}>
          <Row title={t("appearance.startTint")} description={t("appearance.startTintHelp")}>
            <Toggle label={t("appearance.startTint")} checked={startTint === "true"} onChange={(v) => setStartTint(String(v))} />
          </Row>
        </FoldCard>
      )}
    </>
  );
}
