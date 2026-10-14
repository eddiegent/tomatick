import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { OverlayKind } from "../overlay/OverlayView";
import type { AlertEvent, Exercise, TimerSettings } from "../engine/api";
import type { ShortcutAction, Shortcuts } from "./shortcut";

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
  /** Start Tomatick when the user logs in. */
  launch_at_login: boolean;
  /** Look for a newer release on launch and then every day. */
  check_for_updates: boolean;
  shortcuts: Shortcuts;
  /** "system", or the code of a language from /locales. */
  language: string;
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

/** Mirrors `DesktopStatus` in src-tauri/src/lib.rs. */
export interface DesktopStatus {
  shortcuts: { action: ShortcutAction; accelerator: string; error: string | null }[];
  autostart_error: string | null;
}

/** How the desktop integration went the last time it was set up. */
export const desktopApi = {
  status: () => invoke<DesktopStatus>("desktop_status"),
};

/** Mirrors `UpdateStatus` in src-tauri/src/updater.rs. */
export interface UpdateStatus {
  /** The running version. */
  current: string;
  /** A release newer than the running version, if one was found. */
  available: { version: string; url: string } | null;
  checking: boolean;
  /** Unix seconds of the last check that got an answer. */
  last_checked: number | null;
  /** Set when the last check failed. */
  error: string | null;
}

/** Commands for the update check. It only looks: nothing is downloaded. */
export const updateApi = {
  status: () => invoke<UpdateStatus>("update_status"),
  /** Checks now, even when the automatic check is off. */
  check: () => invoke<UpdateStatus>("update_check"),
  /** Opens the page of the newer release in the browser. */
  open: () => invoke<void>("update_open"),
};

/** What the update check knows, kept up to date as checks end. */
export function useUpdateStatus(): UpdateStatus | null {
  const [status, setStatus] = useState<UpdateStatus | null>(null);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;

    listen<UpdateStatus>("update:changed", (e) => setStatus(e.payload)).then((fn) => {
      if (active) {
        unlisten = fn;
      } else {
        fn();
      }
    });
    updateApi
      .status()
      .then((initial) => {
        if (active) {
          setStatus((current) => current ?? initial);
        }
      })
      .catch(() => {});

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return status;
}

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
