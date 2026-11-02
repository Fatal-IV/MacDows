import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { inTauri } from "./api";

if (!inTauri) document.body.classList.add("preview");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
