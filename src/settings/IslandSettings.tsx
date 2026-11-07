import { Children, useState, type ReactNode } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { SCREEN_IDS, setSetting, type ClickMode, type FullscreenMode, type ScreenId, type Settings as AppSettings } from "../bridge";
import { DEFAULT_ORDER } from "../notch/islandDefaults";
import { canDisable } from "../notch/islandLogic";
import { Button, Group, Row, Select, Slider, Toggle } from "../ui/controls";
import { useSystemStatus } from "../system";
import Appearance from "./Appearance";
import { IslandStylePicker } from "./IslandStylePicker";
import { VisibilityPicker } from "./VisibilityPicker";
import { StatusWidgetsEditor } from "./StatusWidgetsEditor";
import { useStored } from "./useStored";

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

/** One screen of the island: a header with its on/off switch, and (while it is on) its own settings, folded away until
 *  the header is clicked. The last screen that is on cannot be switched off, and says why. */
function ScreenSection({ s, t, id, children }: { s: AppSettings; t: T; id: ScreenId; children?: ReactNode }) {
  const on = s.island.screens[id].enabled;
  const locked = on && !canDisable(s.island, id);
  const [open, setOpen] = useState(false);
  const expandable = on && Children.toArray(children).length > 0;
  return (
    <Group>
      <div
        className={`ui-row${expandable ? " ui-row-fold" : ""}`}
        role={expandable ? "button" : undefined}
        tabIndex={expandable ? 0 : undefined}
        aria-expanded={expandable ? open : undefined}
        onClick={expandable ? () => setOpen((v) => !v) : undefined}
        onKeyDown={expandable ? (e) => (e.key === "Enter" || e.key === " ") && e.target === e.currentTarget && (e.preventDefault(), setOpen((v) => !v)) : undefined}
      >
        <div className="ui-row-text">
          <div className="ui-row-title">{t(`island.screen.${id}`)}</div>
          {locked && <div className="ui-row-desc">{t("island.lastScreen")}</div>}
        </div>
        <div className="ui-row-control" onClick={(e) => e.stopPropagation()}>
          {expandable && (
            <svg className={`ui-chevron${open ? " open" : ""}`} width="14" height="14" viewBox="0 0 24 24" aria-hidden onClick={() => setOpen((v) => !v)}>
              <path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          )}
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
        </div>
      </div>
      <AnimatePresence initial={false}>
        {expandable && open && (
          <motion.div key="body" className="ui-fold" initial={{ height: 0, opacity: 0, overflow: "hidden" }} animate={{ height: "auto", opacity: 1, transitionEnd: { overflow: "visible" } }} exit={{ height: 0, opacity: 0, overflow: "hidden" }} transition={{ duration: 0.18 }}>
            {children}
          </motion.div>
        )}
      </AnimatePresence>
    </Group>
  );
}

/** A card that starts folded: its header (title and chevron) opens and closes the rows inside. */
function FoldCard({ title, children }: { title: string; children: ReactNode }) {
  const [open, setOpen] = useState(false);
  const toggle = () => setOpen((v) => !v);
  return (
    <Group>
      <div className="ui-row ui-row-fold" role="button" tabIndex={0} aria-expanded={open} onClick={toggle} onKeyDown={(e) => (e.key === "Enter" || e.key === " ") && (e.preventDefault(), toggle())}>
        <div className="ui-row-text">
          <div className="ui-row-title">{title}</div>
        </div>
        <div className="ui-row-control">
          <svg className={`ui-chevron${open ? " open" : ""}`} width="14" height="14" viewBox="0 0 24 24" aria-hidden>
            <path d="M6 9l6 6 6-6" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
          </svg>
        </div>
      </div>
      <AnimatePresence initial={false}>
        {open && (
          <motion.div key="body" className="ui-fold" initial={{ height: 0, opacity: 0, overflow: "hidden" }} animate={{ height: "auto", opacity: 1, transitionEnd: { overflow: "visible" } }} exit={{ height: 0, opacity: 0, overflow: "hidden" }} transition={{ duration: 0.18 }}>
            {children}
          </motion.div>
        )}
      </AnimatePresence>
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
  const system = useSystemStatus();

  const resetLayout = () => {
    updateIsland(s, (i) => {
      i.style = "notch";
      i.visibility = "fixed";
      i.fullscreen = "edge";
    });
    setScale("1");
  };
  const resetNav = () =>
    updateIsland(s, (i) => {
      i.nav = { wheel: true, order: [...DEFAULT_ORDER], click: "smart", middle: true, remember: false, hover_player: true };
    });

  return (
    <>
      <h2 className="section-title">{t("island.sectionLayout")}</h2>
      <Group>
        <IslandStylePicker value={s.island.style} onChange={(v) => updateIsland(s, (i) => (i.style = v))} t={t} />
        <VisibilityPicker value={s.island.visibility} onChange={(v) => updateIsland(s, (i) => (i.visibility = v))} t={t} islandStyle={s.island.style} />
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

      <h2 className="section-title">{t("island.sectionTheme")}</h2>
      <Appearance />

      <h2 className="section-title">{t("island.sectionScreens")}</h2>
      {SCREEN_IDS.map((id) => (
        <ScreenSection key={id} s={s} t={t} id={id}>
          {id === "quick" && (
            <>
              <Row title={t("island.quick.wifiPage")} description={t("island.quick.wifiPageHelp")}>
                <Toggle label={t("island.quick.wifiPage")} checked={s.island.screens.quick.wifi_page} onChange={(v) => updateIsland(s, (i) => (i.screens.quick.wifi_page = v))} />
              </Row>
              <Row title={t("island.quick.vpn")} description={t("island.quick.vpnHelp")}>
                <Toggle label={t("island.quick.vpn")} checked={s.island.screens.quick.vpn} onChange={(v) => updateIsland(s, (i) => (i.screens.quick.vpn = v))} />
              </Row>
              <Row title={t("island.quick.pairing")} description={t("island.quick.pairingHelp")}>
                <Toggle label={t("island.quick.pairing")} checked={s.island.screens.quick.bluetooth_pairing} onChange={(v) => updateIsland(s, (i) => (i.screens.quick.bluetooth_pairing = v))} />
              </Row>
            </>
          )}
          {id === "status" && (
            <StatusWidgetsEditor
              cfg={s.island.screens.status}
              hasBattery={system?.battery_percent != null}
              t={t}
              onChange={(next) =>
                updateIsland(s, (i) => {
                  i.screens.status = next;
                })
              }
            />
          )}
          {id === "music" && (
            <>
              <Row title={t("island.music.sources")} description={t("island.music.sourcesHelp")}>
                <Toggle label={t("island.music.sources")} checked={s.island.screens.music.sources} onChange={(v) => updateIsland(s, (i) => (i.screens.music.sources = v))} />
              </Row>
              <Row title={t("island.music.gradient")} description={t("island.music.gradientHelp")}>
                <Select
                  label={t("island.music.gradient")}
                  value={String(s.island.screens.music.gradient)}
                  onChange={(v) => updateIsland(s, (i) => (i.screens.music.gradient = Number(v)))}
                  options={Array.from({ length: 9 }, (_, n) => ({ value: String(n), label: t(`island.music.gradient.${n}`) }))}
                />
              </Row>
            </>
          )}
        </ScreenSection>
      ))}

      <FoldCard title={t("island.sectionNav")}>
        <Row title={t("island.nav.wheel")}>
          <Toggle label={t("island.nav.wheel")} checked={s.island.nav.wheel} onChange={(v) => updateIsland(s, (i) => (i.nav.wheel = v))} />
        </Row>
        <Row title={t("island.nav.click")}>
          <Select label={t("island.nav.click")} value={s.island.nav.click} onChange={(v) => updateIsland(s, (i) => (i.nav.click = v as ClickMode))} options={CLICK_MODES.map((m) => ({ value: m, label: t(`island.click.${m}`) }))} />
        </Row>
        <Row title={t("island.nav.hoverPlayer")} description={t("island.nav.hoverPlayerHelp")}>
          <Toggle label={t("island.nav.hoverPlayer")} checked={s.island.nav.hover_player} onChange={(v) => updateIsland(s, (i) => (i.nav.hover_player = v))} />
        </Row>
        <Row title={t("island.nav.middle")}>
          <Toggle label={t("island.nav.middle")} checked={s.island.nav.middle} onChange={(v) => updateIsland(s, (i) => (i.nav.middle = v))} />
        </Row>
        <Row title={t("island.nav.remember")} description={t("island.nav.rememberHelp")}>
          <Toggle label={t("island.nav.remember")} checked={s.island.nav.remember} onChange={(v) => updateIsland(s, (i) => (i.nav.remember = v))} />
        </Row>
        <div className="ui-row">
          <span className="ui-row-title">{t("island.nav.order")}</span>
        </div>
        <OrderList s={s} t={t} />
        <div className="ui-row actions">
          <Button onClick={resetNav}>{t("island.reset")}</Button>
        </div>
      </FoldCard>
    </>
  );
}
