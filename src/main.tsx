import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Notch from "./notch/Notch";

const views: Record<string, () => React.ReactElement> = {
  notch: () => <Notch />,
};

const label = getCurrentWindow().label;
const View = views[label] ?? views.notch;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <View />
  </React.StrictMode>,
);
