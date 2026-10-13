import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import i18n, { resolveLanguage } from "./i18n";
import { useSettings } from "./settings/api";
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

  // The language follows the setting, in every window.
  const language = useSettings()?.settings.language;
  useEffect(() => {
    if (language !== undefined) {
      void i18n.changeLanguage(resolveLanguage(language));
    }
  }, [language]);
  useEffect(() => {
    const apply = (lng: string) => {
      document.documentElement.lang = lng;
    };
    apply(i18n.language);
    i18n.on("languageChanged", apply);
    return () => i18n.off("languageChanged", apply);
  }, []);

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
