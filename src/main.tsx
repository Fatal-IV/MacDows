import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Notch from "./notch/Notch";
import Dock from "./dock/Dock";

const views: Record<string, () => React.ReactElement> = {
  notch: () => <Notch />,
  dock: () => <Dock />,
};

const label = getCurrentWindow().label;
const View = views[label] ?? views.notch;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <View />
  </React.StrictMode>,
);
