import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { closePopup } from "./api";

// A blur that fires while the window is still being focused would close it at once.
const BLUR_GRACE_MS = 400;

/** Closes the popup window on Esc and when it loses focus. */
export function useCloseOnBlur(label: string) {
  useEffect(() => {
    const opened = Date.now();
    const close = () => closePopup(label);
    const un = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (!focused && Date.now() - opened > BLUR_GRACE_MS) close();
    });
    const onKey = (e: KeyboardEvent) => e.key === "Escape" && close();
    window.addEventListener("keydown", onKey);
    return () => {
      un.then((f) => f());
      window.removeEventListener("keydown", onKey);
    };
  }, [label]);
}
