import { useState } from "react";
import { motion } from "framer-motion";
import "./App.css";

const COLLAPSED = { width: 170, height: 32 };
const EXPANDED = { width: 420, height: 130 };

export default function App() {
  const [open, setOpen] = useState(false);
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
          className="content"
          initial={false}
          animate={{ opacity: open ? 1 : 0, y: open ? 0 : -6 }}
          transition={{ duration: 0.18, delay: open ? 0.08 : 0 }}
        >
          <div className="title">Nothing playing</div>
          <div className="sub">Media controls coming soon</div>
        </motion.div>
      </motion.div>
    </div>
  );
}
