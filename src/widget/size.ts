import { useEffect } from "react";
import { LogicalSize } from "@tauri-apps/api/dpi";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { WidgetSize } from "../settings/api";

/** Window edge length in logical pixels. The tomato fills the window. */
export const WIDGET_PIXELS: Record<WidgetSize, number> = {
  small: 140,
  medium: 200,
  large: 280,
};

/** Resizes the widget window whenever the size setting changes. */
export function useApplyWidgetSize(size: WidgetSize): void {
  useEffect(() => {
    const pixels = WIDGET_PIXELS[size];
    try {
      getCurrentWindow()
        .setSize(new LogicalSize(pixels, pixels))
        .catch(() => {});
    } catch {
      // Not running inside Tauri (for example a plain browser).
    }
  }, [size]);
}
