import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import "./Notch.css";

const COLLAPSED = { width: 170, height: 32 };
const EXPANDED = { width: 420, height: 130 };

export default function Notch() {
  const [open, setOpen] = useState(false);
  const [now, setNow] = useState(new Date());
  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000 * 15);
    return () => clearInterval(id);
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
        <motion.div
          className="pill-time"
          initial={false}
          animate={{ opacity: open ? 0 : 1 }}
          transition={{ duration: 0.12 }}
        >
          {time}
        </motion.div>
        <motion.div
          className="content"
          initial={false}
          animate={{ opacity: open ? 1 : 0, y: open ? 0 : -6 }}
          transition={{ duration: 0.18, delay: open ? 0.08 : 0 }}
        >
          <div className="title">{time}</div>
          <div className="sub">Nothing playing</div>
        </motion.div>
      </motion.div>
    </div>
  );
}
