import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { Overlay } from "./overlay/Overlay";
import { SettingsDialog } from "./settings/SettingsDialog";
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
  const overlay = label === "overlay";

  useEffect(() => {
    document.documentElement.classList.toggle("settings-window", settings);
    document.documentElement.classList.toggle("overlay-window", overlay);
  }, [settings, overlay]);

  if (settings) {
    return <SettingsDialog />;
  }
  return overlay ? <Overlay /> : <Widget />;
}

export default App;
