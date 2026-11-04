import type { ReactNode } from "react";
import { SCREEN_IDS, setSetting, type ClickMode, type FullscreenMode, type ScreenId, type Settings as AppSettings, type Visibility } from "../bridge";
import { DEFAULT_ORDER } from "../notch/islandDefaults";
import { canDisable } from "../notch/islandLogic";
import { Button, Group, Row, Select, Slider, Toggle } from "../ui/controls";
import { useStored } from "./useStored";

const VISIBILITIES: Visibility[] = ["fixed", "smart", "peek"];
const FULLSCREEN_MODES: FullscreenMode[] = ["edge", "hidden"];
const CLICK_MODES: ClickMode[] = ["smart", "quick", "player"];

type Island = AppSettings["island"];

/** Changes the island settings; the whole object is written, so nothing else in it is lost. */
export function updateIsland(s: AppSettings, change: (draft: Island) => void) {
  const draft = structuredClone(s.island);
  change(draft);
  return setSetting("island", draft);
}

type T = (key: string) => string;

/** One screen of the island: a header with its on/off switch, and (while it is on) its own settings. The last
 *  screen that is on cannot be switched off, and says why. */
function ScreenSection({ s, t, id, children }: { s: AppSettings; t: T; id: ScreenId; children?: ReactNode }) {
  const on = s.island.screens[id].enabled;
  const locked = on && !canDisable(s.island, id);
  return (
    <Group>
      <Row title={t(`island.screen.${id}`)} description={locked ? t("island.lastScreen") : undefined}>
        <Toggle
          label={t(`island.screen.${id}`)}
          checked={on}
          disabled={locked}
          onChange={(v) =>
            updateIsland(s, (i) => {
              i.screens[id].enabled = v;
            })
          }
        />
      </Row>
      {on && children}
    </Group>
  );
}

/** The order of the screens: each row can move up or down. */
function OrderList({ s, t }: { s: AppSettings; t: T }) {
  const order = s.island.nav.order;
  const move = (index: number, delta: number) =>
    updateIsland(s, (i) => {
      const to = index + delta;
      if (to < 0 || to >= i.nav.order.length) return;
      [i.nav.order[index], i.nav.order[to]] = [i.nav.order[to], i.nav.order[index]];
    });
  return (
    <>
      {order.map((id, index) => (
        <div className="ui-row order-row" key={id}>
          <span className="ui-row-title">{t(`island.screen.${id}`)}</span>
          <span className="ui-row-control">
            <Button disabled={index === 0} onClick={() => move(index, -1)}>
              ↑
            </Button>
            <Button disabled={index === order.length - 1} onClick={() => move(index, 1)}>
              ↓
            </Button>
          </span>
        </div>
      ))}
      <div className="ui-row actions">
        <Button
          onClick={() =>
            updateIsland(s, (i) => {
              i.nav.order = [...DEFAULT_ORDER];
            })
          }
        >
          {t("island.nav.orderReset")}
        </Button>
      </div>
    </>
  );
}

/** The "Dynamic Island" settings page. */
export default function IslandSettings({ s, t }: { s: AppSettings; t: T }) {
  const [scale, setScale] = useStored("nectar-scale", "1");

  const resetLayout = () => {
    updateIsland(s, (i) => {
      i.visibility = "fixed";
      i.fullscreen = "edge";
    });
    setScale("1");
  };
  const resetNav = () =>
    updateIsland(s, (i) => {
      i.nav = { wheel: true, order: [...DEFAULT_ORDER], click: "smart", middle: true, remember: false };
    });

  return (
    <>
      <h2 className="section-title">{t("island.sectionLayout")}</h2>
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
        <div className="ui-row actions">
          <Button onClick={resetLayout}>{t("island.reset")}</Button>
        </div>
      </Group>

      <h2 className="section-title">{t("island.sectionScreens")}</h2>
      {SCREEN_IDS.map((id) => (
        <ScreenSection key={id} s={s} t={t} id={id} />
      ))}

      <h2 className="section-title">{t("island.sectionNav")}</h2>
      <Group>
        <Row title={t("island.nav.wheel")}>
          <Toggle label={t("island.nav.wheel")} checked={s.island.nav.wheel} onChange={(v) => updateIsland(s, (i) => (i.nav.wheel = v))} />
        </Row>
        <Row title={t("island.nav.click")}>
          <Select label={t("island.nav.click")} value={s.island.nav.click} onChange={(v) => updateIsland(s, (i) => (i.nav.click = v as ClickMode))} options={CLICK_MODES.map((m) => ({ value: m, label: t(`island.click.${m}`) }))} />
        </Row>
        <Row title={t("island.nav.middle")}>
          <Toggle label={t("island.nav.middle")} checked={s.island.nav.middle} onChange={(v) => updateIsland(s, (i) => (i.nav.middle = v))} />
        </Row>
        <Row title={t("island.nav.remember")} description={t("island.nav.rememberHelp")}>
          <Toggle label={t("island.nav.remember")} checked={s.island.nav.remember} onChange={(v) => updateIsland(s, (i) => (i.nav.remember = v))} />
        </Row>
      </Group>
      <Group>
        <div className="ui-row">
          <span className="ui-row-title">{t("island.nav.order")}</span>
        </div>
        <OrderList s={s} t={t} />
        <div className="ui-row actions">
          <Button onClick={resetNav}>{t("island.reset")}</Button>
        </div>
      </Group>
    </>
  );
}
