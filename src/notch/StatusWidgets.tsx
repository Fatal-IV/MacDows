import { useEffect, useState } from "react";
import { getSystemStats, type StatusCfg, type SystemStats } from "../bridge";
import type { SystemStatus } from "../system";
import { StatGlyph, WeatherGlyph } from "./widgetIcons";
import { formatRate, formatTemp, isLowBattery, weatherCacheFresh, weatherIcon, type WidgetId } from "./widgetLogic";

const STATS_MS = 2000;
const WEATHER_KEY = "macdows.weather";

export interface Weather {
  temp: number;
  code: number;
}

/** Polls the system numbers every 2 seconds while `active` (the island is hovered and a stats widget is placed). */
export function useStats(active: boolean): SystemStats | null {
  const [stats, setStats] = useState<SystemStats | null>(null);
  useEffect(() => {
    if (!active) return;
    let alive = true;
    const poll = () =>
      getSystemStats()
        .then((s) => alive && setStats(s))
        .catch(() => {});
    poll();
    const id = setInterval(poll, STATS_MS);
    return () => {
      alive = false;
      clearInterval(id);
      // The next hover starts from fresh samples, not from a stale one.
      setStats(null);
    };
  }, [active]);
  return stats;
}

interface Cached extends Weather {
  key: string;
  at: number;
}

const readCache = (key: string): Cached | null => {
  try {
    const c = JSON.parse(localStorage.getItem(WEATHER_KEY) ?? "null") as Cached | null;
    return c && c.key === key ? c : null;
  } catch {
    return null;
  }
};

/** Current weather for the chosen city. No city, no request. The last value is kept when the network fails. */
export function useWeather(cfg: StatusCfg): Weather | null {
  const { weather_city: city, weather_lat: lat, weather_lon: lon } = cfg;
  const key = `${lat},${lon}`;
  const ready = city.trim() !== "" && !(lat === 0 && lon === 0);
  const [weather, setWeather] = useState<Weather | null>(() => (ready ? readCache(key) : null));
  useEffect(() => {
    if (!ready) {
      setWeather(null);
      return;
    }
    let alive = true;
    const load = async () => {
      const cached = readCache(key);
      if (cached && weatherCacheFresh(cached.at, Date.now())) {
        setWeather(cached);
        return;
      }
      try {
        const res = await fetch(`https://api.open-meteo.com/v1/forecast?latitude=${lat}&longitude=${lon}&current=temperature_2m,weather_code`);
        if (!res.ok) throw new Error(String(res.status));
        const cur = (await res.json()).current;
        const next: Cached = { key, at: Date.now(), temp: Number(cur.temperature_2m), code: Number(cur.weather_code) };
        if (!Number.isFinite(next.temp)) throw new Error("bad data");
        localStorage.setItem(WEATHER_KEY, JSON.stringify(next));
        if (alive) setWeather(next);
      } catch {
        if (alive) setWeather(cached);
      }
    };
    load();
    const id = setInterval(load, 5 * 60_000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [ready, key, lat, lon]);
  return ready ? weather : null;
}

interface Props {
  ids: WidgetId[];
  cfg: StatusCfg;
  stats: SystemStats | null;
  weather: Weather | null;
  status: SystemStatus | null;
}

/** One side of the clock. */
export function StatusWidgets({ ids, cfg, stats, weather, status }: Props) {
  return (
    <>
      {ids.map((id) => {
        switch (id) {
          case "weather":
            return (
              weather && (
                <span key={id} className="widget">
                  <WeatherGlyph icon={weatherIcon(weather.code)} />
                  {formatTemp(weather.temp, cfg.temp_unit)}
                </span>
              )
            );
          case "battery": {
            const p = status?.battery_percent;
            if (p == null) return null;
            const low = isLowBattery(p, cfg.low_battery, status?.battery_charging ?? false);
            return (
              <span key={id} className={`widget${low ? " low" : ""}`}>
                <StatGlyph id="battery" />
                {p}%
              </span>
            );
          }
          case "net":
            return (
              <span key={id} className="widget">
                <StatGlyph id="net" />
                {formatRate(stats?.net_down ?? 0)}
              </span>
            );
          default: {
            const v = stats?.[id];
            return (
              <span key={id} className="widget">
                <StatGlyph id={id} />
                {v == null ? "–" : `${v}%`}
              </span>
            );
          }
        }
      })}
    </>
  );
}
