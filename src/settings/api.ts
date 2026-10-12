import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Exercise, TimerSettings } from "../engine/api";

export type WidgetSize = "small" | "medium" | "large";

/** Mirrors `AppSettings` in src-tauri/src/storage.rs. */
export interface AppSettings {
  timer: TimerSettings;
  widget_size: WidgetSize;
  /** Starting value of the countdown timer, in seconds. */
  countdown_secs: number;
}

export interface SettingsPayload {
  settings: AppSettings;
  exercises: Exercise[];
}

/** Commands for the stored settings. */
export const settingsApi = {
  get: () => invoke<SettingsPayload>("settings_get"),
  /** The classic 25/5/15 profile. */
  classic: () => invoke<TimerSettings>("settings_classic"),
  /** Rejects with a message when a value is out of range. */
  save: (settings: AppSettings, exercises: Exercise[]) =>
    invoke<SettingsPayload>("settings_save", { settings, exercises }),
  open: () => invoke<void>("settings_open"),
};

/** The stored settings, kept up to date when they change anywhere. */
export function useSettings(): SettingsPayload | null {
  const [payload, setPayload] = useState<SettingsPayload | null>(null);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;

    listen<SettingsPayload>("settings:changed", (e) => setPayload(e.payload)).then(
      (fn) => {
        if (active) {
          unlisten = fn;
        } else {
          fn();
        }
      },
    );
    settingsApi.get().then((initial) => {
      if (active) {
        setPayload((current) => current ?? initial);
      }
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return payload;
}
