import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { Overlay } from "./overlay/Overlay";
import { SettingsDialog } from "./settings/SettingsDialog";
import { StatsWindow } from "./stats/StatsWindow";
import { Widget } from "./widget/Widget";

/** The same page serves every window; the window label picks the view. */
function windowLabel(): string {
  try {
    return getCurrentWindow().label;
  } catch {
    return "main";
  }
}

function App() {
  const label = windowLabel();
  const settings = label === "settings";
  const stats = label === "stats";
  const overlay = label === "overlay";

  useEffect(() => {
    // The settings and statistics windows share the opaque window theme.
    document.documentElement.classList.toggle("settings-window", settings || stats);
    document.documentElement.classList.toggle("overlay-window", overlay);
  }, [settings, stats, overlay]);

  if (settings) {
    return <SettingsDialog />;
  }
  if (stats) {
    return <StatsWindow />;
  }
  return overlay ? <Overlay /> : <Widget />;
}

export default App;
