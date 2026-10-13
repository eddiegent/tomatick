import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { OverlayKind } from "../overlay/OverlayView";
import type { AlertEvent, Exercise, TimerSettings } from "../engine/api";

export type WidgetSize = "small" | "medium" | "large";

/** The sounds that ship with the app (public/sounds). */
export type BuiltinSound = "chime" | "bell" | "beep" | "marimba";

/** Mirrors `SoundChoice` in src-tauri/src/alerts.rs. */
export type SoundChoice =
  | { kind: "builtin"; value: BuiltinSound }
  /** A file name inside the custom sounds folder of the app data. */
  | { kind: "custom"; value: string };

/** Mirrors `EventAlerts` in src-tauri/src/alerts.rs. */
export interface EventAlerts {
  notification: boolean;
  sound_enabled: boolean;
  sound: SoundChoice;
  /** Stored now, used when the overlay arrives. */
  overlay: boolean;
  bring_to_front: boolean;
}

/** Mirrors `AlertSettings` in src-tauri/src/alerts.rs. */
export interface AlertSettings extends Record<AlertEvent, EventAlerts> {
  /** Volume of every alert sound, 0 to 100. */
  volume: number;
  muted: boolean;
}

/** Mirrors `AppSettings` in src-tauri/src/storage.rs. */
export interface AppSettings {
  timer: TimerSettings;
  widget_size: WidgetSize;
  /** Starting value of the countdown timer, in seconds. */
  countdown_secs: number;
  /** Quick-pick countdown durations, in seconds. One to six of them. */
  countdown_presets: number[];
  /** Pomodoros to reach each day, shown in the statistics. */
  daily_goal: number;
  alerts: AlertSettings;
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

/** Commands for the custom alert sounds. */
export const soundApi = {
  /**
   * Opens a file picker and copies the chosen audio file into the app data.
   * Resolves to the stored file name, or `null` if the dialog was cancelled.
   */
  add: () => invoke<string | null>("sound_add"),
};

export type { OverlayKind };

/** Commands for the full-screen overlay. */
export const overlayApi = {
  /** Opens the overlay with sample content. The timer is not touched. */
  preview: (kind: OverlayKind) => invoke<void>("overlay_preview", { kind }),
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
