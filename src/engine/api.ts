import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// These types mirror the Rust engine (src-tauri/src/engine). Field and
// variant names are snake_case because that is what serde produces.

export type Mode = "pomodoro" | "countdown";
export type EngineState = "idle" | "running" | "paused" | "auto_paused";
export type PhaseKind = "work" | "short_break" | "long_break";
export type PhaseStatus = "completed" | "skipped" | "interrupted";
export type AlertEvent =
  | "work_end"
  | "short_break_end"
  | "long_break_end"
  | "countdown_end";
export type SleepBehavior = "auto_pause" | "keep_running";

export interface SegmentInfo {
  index: number;
  count: number;
  total_ms: number;
  remaining_ms: number;
}

export interface CycleInfo {
  position: number;
  length: number;
}

export interface Snapshot {
  mode: Mode;
  state: EngineState;
  /** `null` in countdown mode. */
  phase: PhaseKind | null;
  total_ms: number;
  remaining_ms: number;
  /** Present during a long break. */
  segment: SegmentInfo | null;
  label: string | null;
  cycle: CycleInfo;
}

export interface PhaseRecord {
  kind: PhaseKind;
  started_unix_ms: number;
  ended_unix_ms: number;
  planned_secs: number;
  actual_secs: number;
  paused_secs: number;
  extended_secs: number;
  status: PhaseStatus;
  labels: string[];
}

export type EngineEvent =
  | { type: "phase_started"; kind: PhaseKind }
  | { type: "segment_changed"; index: number; count: number; label: string | null }
  | ({ type: "phase_recorded" } & PhaseRecord)
  | { type: "alert"; event: AlertEvent; label: string | null };

export interface TimerSettings {
  work_secs: number;
  short_break_secs: number;
  long_break_secs: number;
  sessions_before_long: number;
  auto_start: boolean;
  sleep_behavior: SleepBehavior;
}

export interface Exercise {
  name: string;
  /** Optional duration or repetition count, free text such as "30 s". */
  amount?: string | null;
}

/** Commands sent to the Rust engine. */
export const engine = {
  snapshot: () => invoke<Snapshot>("engine_snapshot"),
  start: () => invoke<void>("engine_start"),
  pause: () => invoke<void>("engine_pause"),
  resume: () => invoke<void>("engine_resume"),
  toggle: () => invoke<void>("engine_toggle"),
  skip: () => invoke<void>("engine_skip"),
  skipAndStart: () => invoke<void>("engine_skip_and_start"),
  reset: () => invoke<void>("engine_reset"),
  extend: (seconds: number) => invoke<void>("engine_extend", { seconds }),
  setMode: (mode: Mode) => invoke<void>("engine_set_mode", { mode }),
  /** Rejects with a message when the duration is out of range. */
  setCountdown: (seconds: number) =>
    invoke<void>("engine_set_countdown", { seconds }),
  /** Rejects with a message when a value is out of range. */
  updateSettings: (settings: TimerSettings) =>
    invoke<void>("engine_update_settings", { settings }),
  setExercises: (exercises: Exercise[]) =>
    invoke<void>("engine_set_exercises", { exercises }),
};

/** Subscribes to engine events (phase recorded, alerts, segment changes). */
export function onEngineEvent(
  handler: (event: EngineEvent) => void,
): Promise<UnlistenFn> {
  return listen<EngineEvent>("engine:event", (e) => handler(e.payload));
}

/** The latest engine state, `null` until the first one arrives. */
export function useSnapshot(): Snapshot | null {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;

    listen<Snapshot>("engine:state", (e) => setSnapshot(e.payload)).then(
      (fn) => {
        if (active) {
          unlisten = fn;
        } else {
          fn();
        }
      },
    );
    engine.snapshot().then((initial) => {
      if (active) {
        setSnapshot((current) => current ?? initial);
      }
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return snapshot;
}

/** Formats milliseconds as `mm:ss`, or `h:mm:ss` from one hour up. */
export function formatClock(ms: number): string {
  const total = Math.max(0, Math.ceil(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(seconds)}`
    : `${pad(minutes)}:${pad(seconds)}`;
}
