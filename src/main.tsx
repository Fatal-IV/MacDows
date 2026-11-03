import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Notch from "./notch/Notch";
import Dock from "./dock/Dock";
import Settings from "./settings/Settings";
import TrayPanel from "./tray/TrayPanel";
import Start from "./launcher/Start";
import Search from "./launcher/Search";

const views: Record<string, () => React.ReactElement> = {
  notch: () => <Notch />,
  dock: () => <Dock />,
  settings: () => <Settings />,
  tray: () => <TrayPanel />,
  start: () => <Start />,
  search: () => <Search />,
};

const label = getCurrentWindow().label;
const View = views[label] ?? views.notch;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <View />
  </React.StrictMode>,
);
