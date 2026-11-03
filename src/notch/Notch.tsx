// The notch's behavior and layout follow Nectar (https://github.com/kua8/Nectar, GPL-3.0):
// hover-to-expand with one damped spring, a status pill and a music pill that swap with the
// mouse wheel or a click on the clock, auto-switching to music when playback starts, and the
// concave "bridge" corners that join the notch to the screen edge.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { listen } from "@tauri-apps/api/event";
import { getMedia, getVolume, mediaNext, mediaPrev, mediaSeek, mediaToggle, onMediaChanged, setVolume, type MediaInfo } from "../bridge";
import { useT } from "../i18n";
import { useUpdateStatus } from "../updates";
import { useSystemStatus } from "../system";
import { initTheme } from "../appearance/theme";
import { SurfaceShape } from "../appearance/SurfaceShape";
import "../appearance/surfaces.css";
import { MusicNoteIcon, PauseIcon, PlayIcon, SkipBackIcon, SkipForwardIcon, SpeakerIcon } from "./icons";
import "./Notch.css";

type Mode = "status" | "music";

// One critically damped spring for width, height and corners so the shape settles together.
const SHELL_SPRING = { type: "spring", stiffness: 400, damping: 40 } as const;
const WHEEL_STEP = 60;
const WHEEL_IDLE_MS = 180;
const MUSIC_IDLE_BACK_MS = 5000;
const MARQUEE_SPEED = 30; // px/s
const MARQUEE_MIN_SECONDS = 5;

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

/** The system output volume. */
function VolumeRow() {
  const [level, setLevel] = useState<number | null>(null);
  const pending = useRef<number | null>(null);
  useEffect(() => {
    getVolume().then((v) => v && setLevel(v.level)).catch(() => {});
  }, []);
  const change = (v: number) => {
    setLevel(v);
    // At most one call in flight while dragging; the last value always goes out.
    if (pending.current !== null) {
      pending.current = v;
      return;
    }
    pending.current = v;
    const send = () => {
      const value = pending.current!;
      setVolume(value).finally(() => {
        if (pending.current === value) pending.current = null;
        else send();
      });
    };
    send();
  };
  if (level === null) return <div className="volume-row" />;
  return (
    <div className="volume-row" onWheel={(e) => change(Math.min(1, Math.max(0, level - Math.sign(e.deltaY) * 0.04)))}>
      <SpeakerIcon size={14} level="low" />
      <div className="slider">
        <input type="range" min="0" max="1" step="0.01" value={level} onChange={(e) => change(parseFloat(e.target.value))} onPointerDown={(e) => e.stopPropagation()} />
        <div className="slider-fill" style={{ width: `${level * 100}%` }} />
      </div>
      <SpeakerIcon size={16} level="high" />
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

  const [hovered, setHovered] = useState(false);
  const [mode, setMode] = useState<Mode>("status");
  const [ready, setReady] = useState(false);
  const [impacted, setImpacted] = useState(false);
  const [media, setMedia] = useState<MediaInfo | null>(null);

  const hasMedia = media !== null;
  const playing = media?.playing ?? false;
  const art = useMemo(() => artUrl(media), [media]);

  // Manual choice sticks: the auto-return to status only applies to music that opened itself.
  const manualMusic = useRef(false);
  const lastTrack = useRef<string | null>(null);
  const lastPlaying = useRef(false);
  const wheel = useRef({ sum: 0, timer: 0 as unknown as ReturnType<typeof setTimeout> });

  useEffect(() => {
    getMedia().then(setMedia).catch(() => setMedia(null));
    const un = onMediaChanged(setMedia);
    return () => {
      un.then((f) => f());
    };
  }, []);

  // Theme (colors, glass style, wallpaper backdrop) and UI scale come from the settings window.
  useEffect(() => initTheme({ backdrop: true }), []);
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
    if (hasMedia && playing && mode === "status" && !hovered && (newTrack || justStarted)) {
      manualMusic.current = false;
      setMode("music");
    }
    lastTrack.current = media?.title ?? null;
    lastPlaying.current = playing;
  }, [hasMedia, playing, media?.title, mode, hovered]);

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

  const available = useCallback((): Mode[] => (hasMedia ? (playing ? ["music", "status"] : ["status", "music"]) : ["status"]), [hasMedia, playing]);

  const cycle = useCallback(
    (direction = 1) => {
      const modes = available();
      const next = modes[(modes.indexOf(mode) + direction + modes.length) % modes.length];
      manualMusic.current = next === "music";
      setMode(next);
    },
    [available, mode],
  );

  const onWheel = (e: React.WheelEvent) => {
    if (!hovered) return;
    const delta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (Math.abs(delta) < 2) return;
    const w = wheel.current;
    w.sum += delta;
    clearTimeout(w.timer);
    w.timer = setTimeout(() => (w.sum = 0), WHEEL_IDLE_MS);
    if (Math.abs(w.sum) < WHEEL_STEP) return;
    const direction = w.sum > 0 ? 1 : -1;
    w.sum = 0;
    cycle(direction);
  };

  const restingMode = (): Mode => (hasMedia && playing ? "music" : "status");

  const onHoverStart = () => {
    setHovered(true);
    setMode(restingMode());
  };
  const onHoverEnd = () => {
    setHovered(false);
    setMode(restingMode());
  };

  const isMusic = hasMedia && mode === "music";
  const battery = system?.battery_percent ?? null;

  const width = !ready ? 34 : isMusic ? (hovered ? 340 : 140 + (playing ? 30 : 0) + 30) : hovered ? (battery != null ? 200 : 170) : 140;
  const height = !ready ? 34 : isMusic && hovered ? (media!.duration_ms > 0 ? 134 : 120) : 36;

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
      >
        <SurfaceShape edge="top" flare={impacted ? 13 : 0} radius={18} edgeRadius={18} />
        {update.kind === "available" && <span className="update-dot" />}

        <AnimatePresence>
          {isMusic && hovered && art && (
            <motion.div className="ambient-glow" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.15 }}>
              <AnimatePresence mode="wait">
                <motion.img key={art} src={art} alt="" draggable={false} initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} transition={{ duration: 0.3 }} />
              </AnimatePresence>
            </motion.div>
          )}
        </AnimatePresence>

        <AnimatePresence mode="wait" initial={false}>
          {isMusic && hovered ? (
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
                <VolumeRow />
              </div>
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
              <div className="side left">{isMusic && <Visualizer playing={playing} />}</div>

              <button className="time-flip" onClick={() => cycle()} title={hasMedia ? t("notch.switchPanel") : undefined}>
                <AnimatePresence mode="wait" initial={false}>
                  <motion.span
                    key={mode}
                    className="time"
                    initial={{ rotateX: -90, opacity: 0 }}
                    animate={{ rotateX: 0, opacity: 1 }}
                    exit={{ rotateX: 90, opacity: 0 }}
                    transition={{ type: "spring", stiffness: 600, damping: 30 }}
                  >
                    <Clock />
                  </motion.span>
                </AnimatePresence>
              </button>

              <div className="side right">
                {isMusic ? (
                  <button
                    className={`art-small${hovered ? " large" : ""}${playing ? "" : " paused"}`}
                    onClick={mediaToggle}
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
                  hovered && battery != null && <span className="battery">{battery}%</span>
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
