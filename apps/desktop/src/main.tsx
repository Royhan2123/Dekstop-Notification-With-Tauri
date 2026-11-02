import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { IslandWindow } from "./windows/island/IslandWindow";
import { SettingsWindow } from "./windows/settings/SettingsWindow";

// Satu bundel untuk semua jendela; label jendela Tauri menentukan yang dirender.
const label = getCurrentWindow().label;
document.documentElement.dataset.window = label; // CSS tiap jendela dibatasi lewat atribut ini
const Root = label === "settings" ? SettingsWindow : IslandWindow;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
