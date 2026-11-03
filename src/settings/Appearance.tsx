import { useState } from "react";
import { STYLE_OPACITY, THEME_STYLES } from "../appearance/theme";
import { useT } from "../i18n";
import { Group, Row, Select, Slider, Toggle } from "../ui/controls";

const MODES = ["auto", "light", "dark", "adaptive"] as const;
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
  const [mode, setMode] = useStored("nectar-theme-mode", "auto");
  const [style, setStyle] = useStored("nectar-theme-style", "solid");
  const [accent, setAccent] = useStored("nectar-theme-accent", "mono");
  const [opacity, setOpacity] = useStored("nectar-theme-opacity", "0.8");
  const [scale, setScale] = useStored("nectar-scale", "1");
  const [startTint, setStartTint] = useStored("macdows-start-tint", "true");

  return (
    <>
      <Group>
        <Row title={t("appearance.mode")}>
          <Select label={t("appearance.mode")} value={mode} onChange={setMode} options={MODES.map((m) => ({ value: m, label: t(`appearance.mode.${m}`) }))} />
        </Row>
        <Row title={t("appearance.style")}>
          <Select
            label={t("appearance.style")}
            value={style}
            onChange={(v) => {
              setStyle(v);
              // Each style is designed around its own opacity; the slider can still change it.
              setOpacity(String(STYLE_OPACITY[v] ?? 0.8));
            }}
            options={THEME_STYLES.map((s) => ({ value: s, label: t(`appearance.style.${s}`) }))}
          />
        </Row>
        <Row title={t("appearance.accent")}>
          <Select label={t("appearance.accent")} value={accent} onChange={setAccent} options={ACCENTS.map((a) => ({ value: a, label: t(`appearance.accent.${a}`) }))} />
        </Row>
      </Group>
      <Group>
        <Row title={t("appearance.opacity")}>
          <Slider label={t("appearance.opacity")} value={parseFloat(opacity)} min={0.3} max={1} step={0.01} onChange={(v) => setOpacity(String(v))} />
          <span className="ui-value">{Math.round(parseFloat(opacity) * 100)}%</span>
        </Row>
        <Row title={t("appearance.scale")}>
          <Slider label={t("appearance.scale")} value={parseFloat(scale)} min={0.8} max={1.2} step={0.05} onChange={(v) => setScale(String(v))} />
          <span className="ui-value">{Math.round(parseFloat(scale) * 100)}%</span>
        </Row>
      </Group>
      <Group>
        <Row title={t("appearance.startTint")} description={t("appearance.startTintHelp")}>
          <Toggle label={t("appearance.startTint")} checked={startTint === "true"} onChange={(v) => setStartTint(String(v))} />
        </Row>
      </Group>
    </>
  );
}
