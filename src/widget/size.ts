import { useEffect, useState } from "react";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type WidgetSize = "small" | "medium" | "large";

/** Window edge length in logical pixels. The tomato fills the window. */
export const WIDGET_PIXELS: Record<WidgetSize, number> = {
  small: 140,
  medium: 200,
  large: 280,
};

// Temporary storage until the settings dialog and database exist
// (milestone 4).
const STORAGE_KEY = "tomatick.widgetSize";

function load(): WidgetSize {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "small" || stored === "medium" || stored === "large") {
      return stored;
    }
  } catch {
    // Storage can be unavailable; fall back to the default.
  }
  return "medium";
}

/** The widget size, applied to the window whenever it changes. */
export function useWidgetSize(): [WidgetSize, (size: WidgetSize) => void] {
  const [size, setSize] = useState<WidgetSize>(load);

  useEffect(() => {
    const pixels = WIDGET_PIXELS[size];
    try {
      getCurrentWindow()
        .setSize(new LogicalSize(pixels, pixels))
        .catch(() => {});
    } catch {
      // Not running inside Tauri (for example a plain browser).
    }
    try {
      localStorage.setItem(STORAGE_KEY, size);
    } catch {
      // Ignore storage failures.
    }
  }, [size]);

  return [size, setSize];
}
