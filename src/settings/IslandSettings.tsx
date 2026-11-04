import { setSetting, type FullscreenMode, type Settings as AppSettings, type Visibility } from "../bridge";
import { Button, Group, Row, Select, Slider } from "../ui/controls";
import { useStored } from "./useStored";

const VISIBILITIES: Visibility[] = ["fixed", "smart", "peek"];
const FULLSCREEN_MODES: FullscreenMode[] = ["edge", "hidden"];

/** Changes the island settings; the whole object is written, so nothing else in it is lost. */
export function updateIsland(s: AppSettings, change: (draft: AppSettings["island"]) => void) {
  const draft = structuredClone(s.island);
  change(draft);
  return setSetting("island", draft);
}

/** The "Dynamic Island" settings page. */
export default function IslandSettings({ s, t }: { s: AppSettings; t: (key: string) => string }) {
  const [scale, setScale] = useStored("nectar-scale", "1");

  const reset = () => {
    updateIsland(s, (i) => {
      i.visibility = "fixed";
      i.fullscreen = "edge";
    });
    setScale("1");
  };

  return (
    <>
      <Group>
        <Row title={t("settings.notch")} description={t("settings.visibilityHelp")}>
          <Select label={t("settings.notch")} value={s.island.visibility} onChange={(v) => updateIsland(s, (i) => (i.visibility = v as Visibility))} options={VISIBILITIES.map((v) => ({ value: v, label: t(`vis.${v}`) }))} />
        </Row>
        <Row title={t("island.fullscreen")} description={t("island.fullscreenHelp")}>
          <Select label={t("island.fullscreen")} value={s.island.fullscreen} onChange={(v) => updateIsland(s, (i) => (i.fullscreen = v as FullscreenMode))} options={FULLSCREEN_MODES.map((m) => ({ value: m, label: t(`island.fullscreen.${m}`) }))} />
        </Row>
        <Row title={t("appearance.scale")}>
          <Slider label={t("appearance.scale")} value={parseFloat(scale)} min={0.8} max={1.2} step={0.05} onChange={(v) => setScale(String(v))} />
          <span className="ui-value">{Math.round(parseFloat(scale) * 100)}%</span>
        </Row>
      </Group>
      <Group>
        <div className="ui-row actions">
          <Button onClick={reset}>{t("island.reset")}</Button>
        </div>
      </Group>
    </>
  );
}
