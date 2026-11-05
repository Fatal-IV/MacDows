import { useEffect, useRef, useState } from "react";
import type { StatusCfg, Widgets } from "../bridge";
import { StatGlyph, WeatherGlyph } from "../notch/widgetIcons";
import { moveWidget, trayWidgets, type WidgetId } from "../notch/widgetLogic";
import { Button, Row, Select, Slider } from "../ui/controls";

type T = (key: string) => string;
type Zone = "left" | "right" | "tray";

export interface CityResult {
  name: string;
  admin: string;
  country: string;
  lat: number;
  lon: number;
}

/** Cities that match `query` (Open-Meteo geocoding). Throws when the service cannot be reached. */
export async function searchCity(query: string): Promise<CityResult[]> {
  const q = query.trim();
  if (q.length < 2) return [];
  const res = await fetch(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=5`);
  if (!res.ok) throw new Error(String(res.status));
  const body = await res.json();
  return ((body.results ?? []) as Record<string, unknown>[]).map((r) => ({
    name: String(r.name ?? ""),
    admin: String(r.admin1 ?? ""),
    country: String(r.country ?? ""),
    lat: Number(r.latitude),
    lon: Number(r.longitude),
  }));
}

const DEFAULT_WIDGETS: Widgets = { left: ["weather"], right: ["battery"] };

function Glyph({ id }: { id: WidgetId }) {
  return id === "weather" ? <WeatherGlyph icon="partly" /> : <StatGlyph id={id} />;
}

interface Drag {
  id: WidgetId;
  x: number;
  y: number;
}

/** The layout editor: a pill preview with a drop area on each side of the clock, and a tray of the unplaced widgets.
 *  Drag with the pointer, or focus a chip and use Alt+arrows. */
export function StatusWidgetsEditor({ cfg, hasBattery, t, onChange }: { cfg: StatusCfg; hasBattery: boolean; t: T; onChange: (next: StatusCfg) => void }) {
  const [drag, setDrag] = useState<Drag | null>(null);
  const [flash, setFlash] = useState<Zone | null>(null);
  const [query, setQuery] = useState(cfg.weather_city);
  const [results, setResults] = useState<CityResult[]>([]);
  const [cityError, setCityError] = useState(false);
  const root = useRef<HTMLDivElement>(null);
  const w = cfg.widgets;

  const apply = (next: Widgets | null, zone: Zone) => {
    if (!next) {
      setFlash(zone);
      setTimeout(() => setFlash(null), 400);
      return;
    }
    onChange({ ...cfg, widgets: next });
  };

  /** Where a pointer at (x, y) would drop: the zone under it and the slot among that zone's chips. */
  const target = (x: number, y: number): { zone: Zone; index: number } | null => {
    const zoneEl = (document.elementFromPoint(x, y) as HTMLElement | null)?.closest<HTMLElement>("[data-zone]");
    if (!zoneEl || !root.current?.contains(zoneEl)) return null;
    const chips = [...zoneEl.querySelectorAll<HTMLElement>("[data-chip]")];
    const index = chips.filter((c) => {
      const r = c.getBoundingClientRect();
      return x > r.left + r.width / 2;
    }).length;
    return { zone: zoneEl.dataset.zone as Zone, index };
  };

  const finish = (id: WidgetId, x: number, y: number) => {
    setDrag(null);
    const to = target(x, y);
    if (!to) return;
    apply(moveWidget(w, id, { side: to.zone, index: to.index }), to.zone);
  };

  const onKey = (e: React.KeyboardEvent, id: WidgetId, zone: Zone) => {
    if (!e.altKey) return;
    const list = zone === "tray" ? [] : w[zone];
    const at = list.indexOf(id);
    let next: Widgets | null | undefined;
    let flashZone: Zone = zone;
    if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
      // Alt+Left moves toward the left area (out of the left area: back to the tray); Alt+Right mirrors it.
      const dest: Zone = e.key === "ArrowLeft" ? (zone === "left" ? "tray" : "left") : zone === "right" ? "tray" : "right";
      flashZone = dest;
      next = moveWidget(w, id, { side: dest, index: dest === "left" ? 99 : 0 });
    } else if ((e.key === "ArrowUp" || e.key === "ArrowDown") && zone !== "tray") {
      next = moveWidget(w, id, { side: zone, index: at + (e.key === "ArrowUp" ? -1 : 1) });
    } else return;
    e.preventDefault();
    apply(next ?? null, flashZone);
  };

  const chip = (id: WidgetId, zone: Zone) => {
    const missing = id === "battery" && !hasBattery;
    return (
      <div
        key={id}
        data-chip
        tabIndex={0}
        className={`wchip${drag?.id === id ? " dragging" : ""}${missing ? " dim" : ""}`}
        title={missing ? t("widget.noBattery") : undefined}
        onPointerDown={(e) => {
          if (e.button !== 0) return;
          e.currentTarget.setPointerCapture(e.pointerId);
          setDrag({ id, x: e.clientX, y: e.clientY });
        }}
        onPointerMove={(e) => drag?.id === id && setDrag({ id, x: e.clientX, y: e.clientY })}
        onPointerUp={(e) => drag?.id === id && finish(id, e.clientX, e.clientY)}
        onPointerCancel={() => setDrag(null)}
        onKeyDown={(e) => onKey(e, id, zone)}
      >
        <Glyph id={id} />
        <span>{t(`widget.name.${id}`)}</span>
      </div>
    );
  };

  // City search: wait for a pause in typing.
  useEffect(() => {
    if (query.trim() === cfg.weather_city || query.trim().length < 2) {
      setResults([]);
      return;
    }
    let alive = true;
    const id = setTimeout(() => {
      searchCity(query)
        .then((r) => alive && (setResults(r), setCityError(false)))
        .catch(() => alive && (setResults([]), setCityError(true)));
    }, 350);
    return () => {
      alive = false;
      clearTimeout(id);
    };
  }, [query, cfg.weather_city]);

  const pick = (c: CityResult) => {
    setQuery(c.name);
    setResults([]);
    onChange({ ...cfg, weather_city: c.name, weather_lat: c.lat, weather_lon: c.lon });
  };

  const zone = (z: "left" | "right") => (
    <div data-zone={z} className={`wzone${flash === z ? " flash" : ""}`} aria-label={t(`widget.${z}`)}>
      {w[z].map((id) => chip(id as WidgetId, z))}
    </div>
  );

  return (
    <div ref={root} className="widget-editor">
      <div className="ui-row-text">
        <div className="ui-row-title">{t("widget.title")}</div>
        <div className="ui-row-desc">{t("widget.help")}</div>
      </div>
      <div className="wpreview">
        {zone("left")}
        <span className="wclock">12:34</span>
        {zone("right")}
      </div>
      <div className="wtray-title">{t("widget.tray")}</div>
      <div data-zone="tray" className="wzone tray">
        {trayWidgets(w).map((id) => chip(id, "tray"))}
      </div>
      {drag && (
        <div className="wghost" style={{ left: drag.x + 8, top: drag.y + 8 }}>
          <Glyph id={drag.id} />
          <span>{t(`widget.name.${drag.id}`)}</span>
        </div>
      )}

      <Row title={t("widget.city")} description={cityError ? t("widget.cityError") : t("widget.cityHelp")}>
        <div className="wcity">
          <input className="ui-input" value={query} placeholder={t("widget.cityPlaceholder")} onChange={(e) => setQuery(e.target.value)} aria-label={t("widget.city")} />
          {results.length > 0 && (
            <ul className="wcity-list">
              {results.map((c, i) => (
                <li key={i}>
                  <button type="button" onClick={() => pick(c)}>
                    {[c.name, c.admin, c.country].filter(Boolean).join(", ")}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
      </Row>
      <Row title={t("widget.unit")}>
        <Select
          label={t("widget.unit")}
          value={cfg.temp_unit}
          onChange={(v) => onChange({ ...cfg, temp_unit: v as StatusCfg["temp_unit"] })}
          options={[
            { value: "celsius", label: t("widget.unit.celsius") },
            { value: "fahrenheit", label: t("widget.unit.fahrenheit") },
          ]}
        />
      </Row>
      <Row title={t("widget.low")}>
        <Slider label={t("widget.low")} value={cfg.low_battery} min={5} max={50} step={5} onChange={(v) => onChange({ ...cfg, low_battery: v })} />
        <span className="ui-value">{cfg.low_battery}%</span>
      </Row>
      <div className="ui-row actions">
        <Button onClick={() => onChange({ ...cfg, widgets: structuredClone(DEFAULT_WIDGETS) })}>{t("widget.reset")}</Button>
      </div>
    </div>
  );
}
