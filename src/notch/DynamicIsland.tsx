// The notch's behavior and layout follow Nectar (https://github.com/kua8/Nectar, GPL-3.0):
// hover-to-expand with one damped spring, a status pill and a music pill that swap with the
// mouse wheel or a click on the clock, auto-switching to music when playback starts, and the
// concave "bridge" corners that join the notch to the screen edge.
import { useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { listen } from "@tauri-apps/api/event";
import { getMedia, getMediaSources, selectMediaSource, mediaNext, mediaPrev, mediaSeek, mediaToggle, onMediaChanged, onNotificationNew, addToShelf, clearShelf, getShelf, removeFromShelf, type MediaInfo, type MediaSource, type Notice, type ShelfItem } from "../bridge";
import { useSettings, useT } from "../i18n";
import { buildStops, clickTarget, restingStop, screenOn, type Stop } from "./islandLogic";
import { defaultIsland } from "./islandDefaults";
import { useUpdateStatus } from "../updates";
import { useSystemStatus } from "../system";
import { initTheme } from "../appearance/theme";
import { SurfaceShape } from "../appearance/SurfaceShape";
import "../appearance/surfaces.css";
import { MusicNoteIcon, PauseIcon, PlayIcon, SkipBackIcon, SkipForwardIcon } from "./icons";
import { ToolsPanel, useTimer } from "./Tools";
import { QuickSettingsScreen } from "./QuickSettings";
import { NotificationBanner } from "./NotificationBanner";
import { DropZone, ShelfScreen } from "./Shelf";
import { NotificationHistory } from "./NotificationHistory";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import "./DynamicIsland.css";

type Mode = "status" | "music";

// One critically damped spring for width, height and corners so the shape settles together.
const SHELL_SPRING = { type: "spring", stiffness: 400, damping: 40 } as const;
const WHEEL_STEP = 60;
const WHEEL_IDLE_MS = 180;
const MUSIC_IDLE_BACK_MS = 5000;
const BANNER_MS = 5000;
const MARQUEE_SPEED = 30; // px/s
const MARQUEE_MIN_SECONDS = 5;

/** The strongest color of the artwork (saturated pixels count more), for the soft gradient behind the player. */
function useArtColor(url: string | null): string | null {
  const [color, setColor] = useState<string | null>(null);
  useEffect(() => {
    if (!url) {
      setColor(null);
      return;
    }
    let dead = false;
    const img = new Image();
    img.onload = () => {
      const c = document.createElement("canvas");
      c.width = c.height = 16;
      const g = c.getContext("2d", { willReadFrequently: true });
      if (!g) return;
      g.drawImage(img, 0, 0, 16, 16);
      const d = g.getImageData(0, 0, 16, 16).data;
      let r = 0, gr = 0, b = 0, w = 0;
      for (let i = 0; i < d.length; i += 4) {
        const sat = Math.max(d[i], d[i + 1], d[i + 2]) - Math.min(d[i], d[i + 1], d[i + 2]);
        const weight = 1 + sat / 16;
        r += d[i] * weight;
        gr += d[i + 1] * weight;
        b += d[i + 2] * weight;
        w += weight;
      }
      if (!dead && w > 0) setColor(`${Math.round(r / w)}, ${Math.round(gr / w)}, ${Math.round(b / w)}`);
    };
    img.src = url;
    return () => {
      dead = true;
    };
  }, [url]);
  return color;
}

const artUrl = (m: MediaInfo | null) => (m?.artwork_png_base64 ? `data:${m.artwork_mime};base64,${m.artwork_png_base64}` : null);

/** The levels of what is playing, in a few frequency bands, from the audio the PC is sending to the speakers. */
function Visualizer({ playing, bars = 5, height = 20 }: { playing: boolean; bars?: number; height?: number }) {
  const [levels, setLevels] = useState<number[]>(() => Array(bars).fill(0));

  useEffect(() => {
    const rest = () => setLevels(Array(bars).fill(0));
    if (!playing) {
      rest();
      return;
    }
    // Frames can arrive faster than the screen refreshes: keep the newest and draw it once per frame.
    let latest: number[] | null = null;
    let frame = 0;
    let lastEvent = Date.now();
    const un = listen<{ bands: number[] }>("audio-visualization", (e) => {
      latest = e.payload.bands;
      lastEvent = Date.now();
      if (!frame) {
        frame = requestAnimationFrame(() => {
          frame = 0;
          if (latest) setLevels(latest.slice(0, bars));
        });
      }
    });
    // Nothing is being sent (paused, or no sound): drop the bars to rest.
    const idle = setInterval(() => Date.now() - lastEvent > 600 && rest(), 300);
    return () => {
      un.then((f) => f());
      clearInterval(idle);
      cancelAnimationFrame(frame);
    };
  }, [playing, bars]);

  return (
    <div className={`visualizer${playing ? " live" : ""}`} style={{ height, width: bars * 6 }} aria-hidden>
      {levels.map((level, i) => (
        <span key={i} className="bar" style={{ transform: `scaleY(${Math.max(0.12, level)})`, opacity: 0.45 + level * 0.5 }} />
      ))}
    </div>
  );
}

/** Long titles scroll to their end and back instead of being cut off. */
function TitleMarquee({ title }: { title: string }) {
  const textRef = useRef<HTMLSpanElement>(null);
  const wrapRef = useRef<HTMLDivElement>(null);
  const [overflow, setOverflow] = useState({ on: false, distance: 0 });

  useEffect(() => {
    const check = () => {
      const text = textRef.current;
      const wrap = wrapRef.current;
      if (!text || !wrap) return;
      const distance = text.scrollWidth - wrap.clientWidth;
      setOverflow({ on: distance > 0, distance: Math.max(distance, 0) });
    };
    check();
    const observer = new ResizeObserver(check);
    if (wrapRef.current) observer.observe(wrapRef.current);
    return () => observer.disconnect();
  }, [title]);

  const seconds = Math.max(overflow.distance / MARQUEE_SPEED, MARQUEE_MIN_SECONDS);
  return (
    <div ref={wrapRef} className="title-wrap">
      <span
        ref={textRef}
        className={`title${overflow.on ? " marquee" : ""}`}
        style={{ "--scroll-distance": `-${overflow.distance}px`, "--scroll-duration": `${seconds}s` } as React.CSSProperties}
      >
        {title}
      </span>
    </div>
  );
}

const clock = (ms: number) => {
  const s = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(s / 3600);
  const mm = String(Math.floor((s % 3600) / 60)).padStart(h ? 2 : 1, "0");
  const ss = String(s % 60).padStart(2, "0");
  return h ? `${h}:${mm}:${ss}` : `${mm}:${ss}`;
};

/** Elapsed and remaining time with a bar; clicking the bar seeks when the player allows it. */
function Progress({ media }: { media: MediaInfo }) {
  const [now, setNow] = useState(Date.now());
  const [dragTo, setDragTo] = useState<number | null>(null);
  // The position just sought to: shown until the player reports a newer one.
  const [sought, setSought] = useState<{ ms: number; at: number } | null>(null);
  useEffect(() => {
    if (!media.playing) return;
    const id = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(id);
  }, [media.playing]);

  const sinceUpdate = media.playing && media.updated_at_ms > 0 ? Math.max(0, now - media.updated_at_ms) : 0;
  const fromSeek = sought !== null && media.updated_at_ms < sought.at;
  const base = fromSeek ? sought.ms + (media.playing ? Math.max(0, now - sought.at) : 0) : media.position_ms + sinceUpdate;
  const elapsed = Math.min(media.duration_ms, dragTo ?? base);
  const ratio = media.duration_ms > 0 ? elapsed / media.duration_ms : 0;

  const at = (e: React.PointerEvent<HTMLDivElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    return Math.round(Math.min(1, Math.max(0, (e.clientX - r.left) / r.width)) * media.duration_ms);
  };
  const canSeek = media.seek_enabled && media.duration_ms > 0;
  return (
    <div className="progress">
      <span className="time-text">{clock(elapsed)}</span>
      <div
        className={`bar-track${canSeek ? " seekable" : ""}`}
        onPointerDown={(e) => {
          if (!canSeek) return;
          e.currentTarget.setPointerCapture(e.pointerId);
          setDragTo(at(e));
        }}
        onPointerMove={(e) => dragTo !== null && setDragTo(at(e))}
        onPointerUp={(e) => {
          if (dragTo === null) return;
          const target = at(e);
          mediaSeek(target);
          setSought({ ms: target, at: Date.now() });
          setDragTo(null);
        }}
      >
        <div className="bar-fill" style={{ width: `${ratio * 100}%` }} />
      </div>
      <span className="time-text">-{clock(media.duration_ms - elapsed)}</span>
    </div>
  );
}

function Art({ url, className }: { url: string | null; className?: string }) {
  return (
    <AnimatePresence mode="wait" initial={false}>
      {url ? (
        <motion.img
          key={url}
          className={className}
          src={url}
          alt=""
          draggable={false}
          initial={{ rotateY: 90, opacity: 0 }}
          animate={{ rotateY: 0, opacity: 1 }}
          exit={{ rotateY: -90, opacity: 0 }}
          transition={{ duration: 0.25, ease: "easeInOut" }}
        />
      ) : (
        <motion.div
          key="placeholder"
          className="art-placeholder"
          initial={{ rotateY: 90, opacity: 0 }}
          animate={{ rotateY: 0, opacity: 1 }}
          exit={{ rotateY: -90, opacity: 0 }}
          transition={{ duration: 0.25, ease: "easeInOut" }}
        >
          <MusicNoteIcon size={28} />
        </motion.div>
      )}
    </AnimatePresence>
  );
}

// UI scale 0.8-1.2, kept small enough that the notch still fits its window.
const readScale = () => Math.min(1.2, Math.max(0.8, parseFloat(localStorage.getItem("nectar-scale") || "1") || 1));

export default function Notch() {
  const t = useT();
  const update = useUpdateStatus();
  const system = useSystemStatus();
  // The island settings (screens, navigation); the defaults until they have loaded.
  const settings = useSettings();
  const island = useMemo(() => settings?.island ?? defaultIsland(), [settings]);
  const musicOn = screenOn(island, "music");
  const statusOn = screenOn(island, "status");
  // The drop handler lives in an effect that is set up once; it asks this instead of the settings.
  const shelfOnRef = useRef(true);
  shelfOnRef.current = screenOn(island, "shelf");

  const [hovered, setHovered] = useState(false);
  const [mode, setMode] = useState<Mode>("status");
  const [ready, setReady] = useState(false);
  const [impacted, setImpacted] = useState(false);
  const [media, setMedia] = useState<MediaInfo | null>(null);
  // A click opens the player; the middle button or the arrow opens calendar | Pomodoro.
  const [open, setOpen] = useState(false);
  const [tools, setTools] = useState(false);
  // A click opens the quick settings: its own screen, separate from the player and the tools.
  const [quick, setQuick] = useState(false);
  // The newest notification, shown for a few seconds (and while the pointer rests on it).
  const [banner, setBanner] = useState<Notice | null>(null);
  // The shelf: files dropped on the notch. `dropping` is true while a file is being dragged over it.
  const [shelf, setShelf] = useState<ShelfItem[]>([]);
  const [shelfOpen, setShelfOpen] = useState(false);
  const [dropping, setDropping] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);
  const timer = useTimer();

  const hasMedia = media !== null;
  const playing = media?.playing ?? false;
  const art = useMemo(() => artUrl(media), [media]);
  const artColor = useArtColor(art);

  // Manual choice sticks: the auto-return to status only applies to music that opened itself.
  const manualMusic = useRef(false);
  const lastTrack = useRef<string | null>(null);
  const lastPlaying = useRef(false);
  const wheel = useRef({ sum: 0, timer: 0 as unknown as ReturnType<typeof setTimeout> });

  useEffect(() => {
    getShelf().then(setShelf).catch(() => {});
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (!shelfOnRef.current) return;
      const d = e.payload;
      if (d.type === "enter" || d.type === "over") setDropping(true);
      else if (d.type === "leave") setDropping(false);
      else if (d.type === "drop") {
        setDropping(false);
        if (d.paths.length === 0) return;
        addToShelf(d.paths).then((items) => {
          setShelf(items);
          setBanner(null);
          setQuick(false);
          setTools(false);
          setOpen(false);
          setHistoryOpen(false);
          setShelfOpen(true);
        }).catch(() => {});
      }
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  // Files may have moved since the shelf was last looked at.
  useEffect(() => {
    if (shelfOpen) getShelf().then(setShelf).catch(() => {});
  }, [shelfOpen]);

  useEffect(() => {
    const un = onNotificationNew(setBanner);
    return () => {
      un.then((f) => f());
    };
  }, []);
  useEffect(() => {
    if (!banner || hovered) return;
    const id = setTimeout(() => setBanner(null), BANNER_MS);
    return () => clearTimeout(id);
  }, [banner, hovered]);

  useEffect(() => {
    getMedia().then(setMedia).catch(() => setMedia(null));
    const un = onMediaChanged(setMedia);
    return () => {
      un.then((f) => f());
    };
  }, []);

  // The browser's own right-click menu makes no sense on the island.
  useEffect(() => {
    const block = (e: MouseEvent) => e.preventDefault();
    document.addEventListener("contextmenu", block);
    return () => document.removeEventListener("contextmenu", block);
  }, []);

  // Theme (colors, glass style, wallpaper backdrop) and UI scale come from the settings window.
  useEffect(() => initTheme({ backdrop: true, syncSystem: true }), []);
  const [scale, setScale] = useState(readScale);
  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === "nectar-scale") setScale(readScale());
    };
    window.addEventListener("storage", onStorage);
    return () => window.removeEventListener("storage", onStorage);
  }, []);

  // Drop-in on startup: the notch slides in, then its top corners flatten into the screen edge.
  useEffect(() => {
    const a = setTimeout(() => setReady(true), 80);
    const b = setTimeout(() => setImpacted(true), 320);
    return () => {
      clearTimeout(a);
      clearTimeout(b);
    };
  }, []);

  // A new track, or playback resuming, brings the music pill up while the pointer is away.
  useEffect(() => {
    const newTrack = playing && (media?.title ?? null) !== lastTrack.current;
    const justStarted = playing && !lastPlaying.current;
    if (musicOn && hasMedia && playing && mode === "status" && !hovered && (newTrack || justStarted)) {
      manualMusic.current = false;
      setMode("music");
    }
    lastTrack.current = media?.title ?? null;
    lastPlaying.current = playing;
  }, [musicOn, hasMedia, playing, media?.title, mode, hovered]);

  // Music that opened itself closes again five seconds after playback stops.
  useEffect(() => {
    if (playing || mode !== "music" || manualMusic.current) return;
    const id = setTimeout(() => setMode("status"), MUSIC_IDLE_BACK_MS);
    return () => clearTimeout(id);
  }, [playing, mode]);

  // Without media there is nothing for the music pill to show.
  useEffect(() => {
    if (!hasMedia && mode === "music") setMode("status");
  }, [hasMedia, mode]);

  const onWheel = (e: React.WheelEvent) => {
    if (!hovered || !island.nav.wheel) return;
    // A list that can still scroll keeps the wheel for itself.
    const list = (e.target as HTMLElement).closest(".history-list");
    if (list && list.scrollHeight > list.clientHeight) return;
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (Math.abs(delta) < 2) return;
    const w = wheel.current;
    w.sum += delta;
    clearTimeout(w.timer);
    w.timer = setTimeout(() => (w.sum = 0), WHEEL_IDLE_MS);
    if (Math.abs(w.sum) < WHEEL_STEP) return;
    const direction = w.sum > 0 ? 1 : -1;
    w.sum = 0;
    go(direction);
  };

  // The wheel walks through the screens that are on, in the order of the settings: the pill (status, music), the
  // player, tools, quick settings, notification history and the shelf. Down goes to the next one, up to the previous.
  const stops = (): Stop[] => buildStops(island, hasMedia);
  const currentStop = (): Stop => (shelfOpen ? "shelf" : historyOpen ? "history" : quick ? "quick" : tools ? "tools" : musicOpen ? "player" : mode);
  const goTo = (stop: Stop) => {
    setOpen(false);
    setTools(false);
    setQuick(false);
    setShelfOpen(false);
    setHistoryOpen(false);
    setBanner(null);
    if (stop === "tools") setTools(true);
    else if (stop === "quick") setQuick(true);
    else if (stop === "history") setHistoryOpen(true);
    else if (stop === "shelf") setShelfOpen(true);
    else if (stop === "player") {
      manualMusic.current = true;
      setMode("music");
      setOpen(true);
    } else {
      manualMusic.current = stop === "music";
      setMode(stop);
    }
  };
  const go = (direction: number) => {
    const list = stops();
    const at = list.indexOf(currentStop());
    const next = list[Math.min(list.length - 1, Math.max(0, (at < 0 ? 0 : at) + direction))];
    if (next !== undefined && next !== currentStop()) goTo(next);
  };

  const restingMode = (): Mode => restingStop(island, hasMedia, playing);

  // Where the island was when the pointer left, for "return to the last screen".
  const lastStop = useRef<Stop | null>(null);

  const onHoverStart = () => {
    setHovered(true);
    const back = lastStop.current;
    if (island.nav.remember && back && stops().includes(back)) goTo(back);
    // With media around, hovering opens the player by itself (the click is for the other screens).
    else if (island.nav.hover_player && musicOn && hasMedia) goTo("player");
    else setMode(restingMode());
  };
  const onHoverEnd = () => {
    lastStop.current = currentStop();
    setHovered(false);
    setOpen(false);
    setTools(false);
    setQuick(false);
    setShelfOpen(false);
    setHistoryOpen(false);
    setMode(restingMode());
  };

  const isMusic = hasMedia && mode === "music";
  const musicOpen = isMusic && open && !tools && !quick && !shelfOpen && !historyOpen && !dropping;
  // The programs that play media; with two or more, the player shows a button for each.
  const sourcesOn = island.screens.music.sources;
  const [sources, setSources] = useState<MediaSource[]>([]);
  useEffect(() => {
    if (!musicOpen || !sourcesOn) return;
    let alive = true;
    const load = () => getMediaSources().then((list) => alive && setSources(list)).catch(() => {});
    load();
    const id = setInterval(load, 2000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [musicOpen, sourcesOn]);
  const showSources = musicOpen && sourcesOn && sources.length >= 2;
  const pickSource = (src: MediaSource) => {
    selectMediaSource(src.id).catch(() => {});
    // The player switches right away; the buttons follow a moment later.
    setTimeout(() => getMediaSources().then(setSources).catch(() => {}), 300);
  };
  const bannerOpen = banner !== null && !open && !tools && !quick && !shelfOpen && !historyOpen && !dropping;
  const toggleTools = () => {
    setTools((v) => !v);
    setOpen(false);
    setQuick(false);
  };
  const openQuick = () => {
    if (open || tools || quick || shelfOpen || historyOpen) return;
    // The click mode decides (the player while media plays, quick settings, or always the player); a target that is off
    // gives way to the next screen that is on, and with none there is nothing to open.
    const target = clickTarget(island, hasMedia, playing);
    if (!target) return;
    setBanner(null);
    goTo(target);
  };
  useEffect(() => {
    if (!hasMedia) setOpen(false);
  }, [hasMedia]);
  const battery = system?.battery_percent ?? null;

  const width = !ready ? 34 : tools || quick || shelfOpen || historyOpen ? 520 : dropping ? 380 : bannerOpen ? 380 : musicOpen ? 340 : isMusic ? 200 : hovered ? (battery != null ? 200 : 170) : hasMedia ? 170 : 140;
  const height = !ready ? 34 : tools ? 280 : quick ? 312 : shelfOpen || historyOpen ? 312 : dropping ? 110 : bannerOpen ? 76 : musicOpen ? (media!.duration_ms > 0 ? 126 : 108) + (showSources ? 36 : 0) : 36;

  return (
    <div className="screen" style={{ zoom: scale }}>
      <motion.div
        className={`nectar${hovered ? " expanded" : ""}${impacted ? " is-impacted" : ""}`}
        initial={{ y: -48, width: 34, height: 34, opacity: 0, borderBottomLeftRadius: 18, borderBottomRightRadius: 18 }}
        animate={{
          y: ready ? 0 : -48,
          width,
          height,
          opacity: ready ? 1 : 0,
          borderTopLeftRadius: impacted ? 0 : 18,
          borderTopRightRadius: impacted ? 0 : 18,
          borderBottomLeftRadius: 18,
          borderBottomRightRadius: 18,
        }}
        transition={{
          width: SHELL_SPRING,
          height: SHELL_SPRING,
          borderTopLeftRadius: SHELL_SPRING,
          borderTopRightRadius: SHELL_SPRING,
          y: { type: "spring", stiffness: 550, damping: 45, mass: 0.8 },
          opacity: { duration: 0.2 },
        }}
        style={{ originY: 0 }}
        onHoverStart={onHoverStart}
        onHoverEnd={onHoverEnd}
        onWheel={onWheel}
        onClick={openQuick}
        onMouseDown={(e) => e.button === 1 && e.preventDefault()}
        onAuxClick={(e) => {
          if (e.button !== 1) return;
          e.preventDefault();
          if (!island.nav.middle || !screenOn(island, "tools")) return;
          toggleTools();
        }}
      >
        <SurfaceShape edge="top" flare={impacted ? 13 : 0} radius={18} edgeRadius={18} />
        {update.kind === "available" && <span className="update-dot" />}

        <AnimatePresence>
          {musicOpen && art && (
            <motion.div className="ambient-glow" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.15 }}>
              <AnimatePresence mode="wait">
                <motion.img key={art} src={art} alt="" draggable={false} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.3 }} />
              </AnimatePresence>
            </motion.div>
          )}
        </AnimatePresence>

        <AnimatePresence>
          {musicOpen && artColor && (
            <motion.div
              className="music-gradient"
              style={{ background: `linear-gradient(135deg, rgba(${artColor}, 0.55) 0%, rgba(${artColor}, 0.18) 55%, transparent 100%)` }}
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.3 }}
            />
          )}
        </AnimatePresence>

        <AnimatePresence mode="wait" initial={false}>
          {dropping ? (
            <motion.div key="drop" className="tools-wrap" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0, transition: { duration: 0.1 } }}>
              <DropZone />
            </motion.div>
          ) : historyOpen ? (
            <motion.div key="history" className="tools-wrap" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.98, transition: { duration: 0.1 } }} transition={{ type: "spring", stiffness: 500, damping: 30 }}>
              <NotificationHistory />
              <button
                className="corner-arrow"
                onClick={(e) => {
                  e.stopPropagation();
                  setHistoryOpen(false);
                }}
                aria-label={t("notch.back")}
              >
                ⌄
              </button>
            </motion.div>
          ) : shelfOpen ? (
            <motion.div key="shelf" className="tools-wrap" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.98, transition: { duration: 0.1 } }} transition={{ type: "spring", stiffness: 500, damping: 30 }}>
              <ShelfScreen
                items={shelf}
                onRemove={(path) => removeFromShelf(path).then(setShelf).catch(() => {})}
                onClear={() => clearShelf().then(() => setShelf([])).catch(() => {})}
              />
              <button
                className="corner-arrow"
                onClick={(e) => {
                  e.stopPropagation();
                  setShelfOpen(false);
                }}
                aria-label={t("notch.back")}
              >
                ⌄
              </button>
            </motion.div>
          ) : bannerOpen ? (
            <motion.div key="banner" className="tools-wrap" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.98, transition: { duration: 0.1 } }} transition={{ type: "spring", stiffness: 500, damping: 30 }}>
              <NotificationBanner
                notice={banner!}
                onDismiss={() => setBanner(null)}
                onOpen={() => {
                  setBanner(null);
                  if (screenOn(island, "notifications")) setHistoryOpen(true);
                }}
              />
            </motion.div>
          ) : quick ? (
            <motion.div key="quick" className="tools-wrap" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.98, transition: { duration: 0.1 } }} transition={{ type: "spring", stiffness: 500, damping: 30 }}>
              <QuickSettingsScreen />
              <button
                className="corner-arrow"
                onClick={(e) => {
                  e.stopPropagation();
                  setQuick(false);
                }}
                aria-label={t("notch.back")}
              >
                ⌄
              </button>
            </motion.div>
          ) : tools ? (
            <motion.div key="tools" className="tools-wrap" initial={{ opacity: 0, scale: 0.98 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.98, transition: { duration: 0.1 } }} transition={{ type: "spring", stiffness: 500, damping: 30 }}>
              <ToolsPanel timer={timer} />
              <button
                className="corner-arrow"
                onClick={(e) => {
                  e.stopPropagation();
                  setTools(false);
                  setOpen(hasMedia);
                }}
                aria-label={t("notch.back")}
              >
                ⌄
              </button>
            </motion.div>
          ) : musicOpen ? (
            <motion.div
              key="music-expanded"
              className="music-expanded"
              initial={{ opacity: 0, scale: 0.98 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.98, transition: { duration: 0.1 } }}
              transition={{ type: "spring", stiffness: 500, damping: 30 }}
            >
              <div className="art-large">
                <Art url={art} />
              </div>
              {screenOn(island, "tools") && (
                <button
                  className="corner-arrow"
                  onClick={(e) => {
                    e.stopPropagation();
                    toggleTools();
                  }}
                  aria-label={t("notch.openTools")}
                >
                  ⌃
                </button>
              )}
              <div className="meta-col">
                <div className="track-row">
                  <div className="track-info">
                    <TitleMarquee title={media!.title} />
                    <span className="artist">{media!.artist}</span>
                  </div>
                  <Visualizer playing={playing} height={18} />
                </div>
                <div className="controls">
                  <motion.button className="ctl" onClick={mediaPrev} aria-label={t("media.previous")} whileHover={{ scale: 1.2 }} whileTap={{ scale: 0.9 }}>
                    <SkipBackIcon size={32} />
                  </motion.button>
                  <motion.button className="ctl play" onClick={mediaToggle} aria-label={t("media.playPause")} whileHover={{ scale: 1.2 }} whileTap={{ scale: 0.95 }}>
                    <AnimatePresence mode="wait" initial={false}>
                      <motion.span key={playing ? "pause" : "play"} className="swap" initial={{ opacity: 0, scale: 0.8 }} animate={{ opacity: 1, scale: 1 }} exit={{ opacity: 0, scale: 0.8 }} transition={{ duration: 0.15 }}>
                        {playing ? <PauseIcon size={26} /> : <PlayIcon size={26} />}
                      </motion.span>
                    </AnimatePresence>
                  </motion.button>
                  <motion.button className="ctl" onClick={mediaNext} aria-label={t("media.next")} whileHover={{ scale: 1.2 }} whileTap={{ scale: 0.9 }}>
                    <SkipForwardIcon size={32} />
                  </motion.button>
                </div>
                {media!.duration_ms > 0 && <Progress media={media!} />}
              </div>
              {showSources && (
                <div className="source-row" onClick={(e) => e.stopPropagation()}>
                  {sources.map((src) => (
                    <button key={src.id} className={`source-btn${src.selected ? " on" : ""}${src.playing ? "" : " paused"}`} title={src.name} onClick={() => pickSource(src)}>
                      {src.icon_png_base64 && <img src={`data:image/png;base64,${src.icon_png_base64}`} alt="" draggable={false} />}
                      <span className="source-name">{src.name}</span>
                      <span className="source-state" aria-hidden>
                        {src.playing ? "▶" : "❚❚"}
                      </span>
                    </button>
                  ))}
                </div>
              )}
            </motion.div>
          ) : (
            <motion.div
              key="pill"
              className="pill"
              initial={{ opacity: 0, y: -5 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: 5, transition: { duration: 0.1 } }}
              transition={{ duration: 0.2 }}
            >
              <div className="side left">{hasMedia && <Visualizer playing={playing} />}</div>

              <button className="time-flip" title={hasMedia ? t("notch.switchPanel") : undefined}>
                <AnimatePresence mode="wait" initial={false}>
                  <motion.span
                    key={mode}
                    className="time"
                    initial={{ rotateX: -90, opacity: 0 }}
                    animate={{ rotateX: 0, opacity: 1 }}
                    exit={{ rotateX: 90, opacity: 0 }}
                    transition={{ type: "spring", stiffness: 600, damping: 30 }}
                  >
                    {statusOn && <Clock />}
                  </motion.span>
                </AnimatePresence>
              </button>

              <div className="side right">
                {isMusic ? (
                  <button
                    className={`art-small${hovered ? " large" : ""}${playing ? "" : " paused"}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      mediaToggle();
                    }}
                    onDoubleClick={mediaNext}
                    onContextMenu={(e) => {
                      e.preventDefault();
                      mediaPrev();
                    }}
                    aria-label={t("media.playPause")}
                  >
                    <Art url={art} />
                    <span className="art-overlay">{playing ? <PauseIcon /> : <PlayIcon />}</span>
                  </button>
                ) : (
                  statusOn && hovered && battery != null && <span className="battery">{battery}%</span>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}

function Clock() {
  const format = () => new Date().toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  const [text, setText] = useState(format);
  useEffect(() => {
    let id: ReturnType<typeof setTimeout>;
    const tick = () => {
      setText(format());
      id = setTimeout(tick, 1000 - (Date.now() % 1000));
    };
    id = setTimeout(tick, 1000 - (Date.now() % 1000));
    return () => clearTimeout(id);
  }, []);
  return <>{text}</>;
}
