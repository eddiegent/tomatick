import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./App.css";
import { SettingsDialog } from "./settings/SettingsDialog";
import { Widget } from "./widget/Widget";

/** The same page serves both windows; the window label picks the view. */
function isSettingsWindow(): boolean {
  try {
    return getCurrentWindow().label === "settings";
  } catch {
    return false;
  }
}

function App() {
  const settings = isSettingsWindow();

  useEffect(() => {
    document.documentElement.classList.toggle("settings-window", settings);
  }, [settings]);

  return settings ? <SettingsDialog /> : <Widget />;
}

export default App;
