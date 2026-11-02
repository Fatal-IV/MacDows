import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { getMedia, mediaNext, mediaPrev, mediaToggle, onMediaChanged, type MediaInfo } from "../bridge";
import { useT } from "../i18n";
import { useUpdateStatus } from "../updates";
import { useSystemStatus } from "../system";
import "./Notch.css";

const COLLAPSED = { width: 170, height: 32 };
const EXPANDED = { width: 420, height: 130 };

export default function Notch() {
  const t = useT();
  const update = useUpdateStatus();
  const system = useSystemStatus();
  const [open, setOpen] = useState(false);
  const [now, setNow] = useState(new Date());
  const [media, setMedia] = useState<MediaInfo | null>(null);

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000 * 15);
    return () => clearInterval(id);
  }, []);

  useEffect(() => {
    getMedia().then(setMedia).catch(() => setMedia(null));
    const un = onMediaChanged(setMedia);
    return () => {
      un.then((f) => f());
    };
  }, []);

  const time = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" }).format(now);
  const size = open ? EXPANDED : COLLAPSED;

  return (
    <div className="stage">
      <motion.div
        className="notch"
        initial={false}
        animate={{
          width: size.width,
          height: size.height,
          borderBottomLeftRadius: open ? 32 : 16,
          borderBottomRightRadius: open ? 32 : 16,
        }}
        transition={{ type: "spring", stiffness: 380, damping: 30, mass: 0.8 }}
        onHoverStart={() => setOpen(true)}
        onHoverEnd={() => setOpen(false)}
      >
        {update.kind === "available" && <span className="update-dot" />}
        <motion.div
          className="pill-time"
          initial={false}
          animate={{ opacity: open ? 0 : 1 }}
          transition={{ duration: 0.12 }}
        >
          {time}
          {system?.battery_percent != null && <span className="pill-battery">{system.battery_percent}%</span>}
        </motion.div>
        <motion.div
          className="content"
          initial={false}
          animate={{ opacity: open ? 1 : 0, y: open ? 0 : -6 }}
          transition={{ duration: 0.18, delay: open ? 0.08 : 0 }}
        >
          {media ? (
            <div className="media">
              {media.artwork_png_base64 ? (
                <img className="art" src={`data:${media.artwork_mime};base64,${media.artwork_png_base64}`} alt="" draggable={false} />
              ) : (
                <div className="art art-empty">♪</div>
              )}
              <div className="meta">
                <div className="title">{media.title}</div>
                <div className="sub">{media.artist || time}</div>
                <div className="controls">
                  <button onClick={mediaPrev} aria-label={t("media.previous")}>⏮</button>
                  <button onClick={mediaToggle} aria-label={t("media.playPause")}>{media.playing ? "⏸" : "▶"}</button>
                  <button onClick={mediaNext} aria-label={t("media.next")}>⏭</button>
                </div>
              </div>
            </div>
          ) : (
            <div className="meta">
              <div className="title">{time}</div>
              <div className="sub">{t("notch.nothingPlaying")}</div>
            </div>
          )}
        </motion.div>
      </motion.div>
    </div>
  );
}
