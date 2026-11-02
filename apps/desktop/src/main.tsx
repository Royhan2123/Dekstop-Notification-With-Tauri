import React from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { IslandWindow } from "./windows/island/IslandWindow";
import { ReviewWindow } from "./windows/review/ReviewWindow";
import { SettingsWindow } from "./windows/settings/SettingsWindow";

// Satu bundel untuk semua jendela; label jendela Tauri menentukan yang dirender.
const label = getCurrentWindow().label;
document.documentElement.dataset.window = label; // CSS tiap jendela dibatasi lewat atribut ini
const ROOTS: Record<string, () => React.JSX.Element | null> = {
  settings: SettingsWindow,
  review: ReviewWindow,
};
const Root = ROOTS[label] ?? IslandWindow;

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
