import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Period = "day" | "week" | "month";

/** Mirrors `Stats` in src-tauri/src/history.rs. */
export interface Stats {
  period: Period;
  /** First and last day of the period, `YYYY-MM-DD`. */
  start: string;
  end: string;
  today: { completed: number; focus_secs: number; goal: number };
  /** One bar per hour for a day (`"00"` to `"23"`), one per date otherwise. */
  buckets: { key: string; completed: number; focus_secs: number }[];
  streak: { current: number; longest: number };
  /** Work phases of the period by how they ended. */
  completion: { completed: number; skipped: number; interrupted: number };
  /** Completed pomodoros of the period by the hour they started. */
  hours: number[];
  totals: {
    completed: number;
    focus_secs: number;
    active_days: number;
    average_focus_secs: number;
  };
}

/**
 * The first day of the week for the language of the app, 0 for Sunday to 6 for
 * Saturday. Falls back to Monday when the webview cannot tell.
 */
export function localeWeekStart(language: string): number {
  try {
    const locale = new Intl.Locale(language) as Intl.Locale & {
      getWeekInfo?: () => { firstDay: number };
      weekInfo?: { firstDay: number };
    };
    const info = locale.getWeekInfo?.() ?? locale.weekInfo;
    if (info) {
      // Intl counts from Monday (1) to Sunday (7).
      return info.firstDay % 7;
    }
  } catch {
    // Fall through to the default.
  }
  return 1;
}

export const statsApi = {
  get: (period: Period, offset: number, weekStart: number) =>
    invoke<Stats>("stats_get", { period, offset, weekStart }),
  open: () => invoke<void>("stats_open"),
};

/**
 * The statistics of a period, refreshed when a phase is added to the
 * history. `null` until the first answer arrives.
 */
export function useStats(
  period: Period,
  offset: number,
  language: string,
): {
  stats: Stats | null;
  error: string | null;
} {
  const [stats, setStats] = useState<Stats | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [refresh, setRefresh] = useState(0);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;
    listen("history:changed", () => setRefresh((n) => n + 1)).then((fn) => {
      if (active) {
        unlisten = fn;
      } else {
        fn();
      }
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  const load = useCallback(() => {
    let active = true;
    statsApi
      .get(period, offset, localeWeekStart(language))
      .then((next) => {
        if (active) {
          setStats(next);
          setError(null);
        }
      })
      .catch((e) => {
        if (active) {
          setError(String(e));
        }
      });
    return () => {
      active = false;
    };
  }, [period, offset, language]);

  useEffect(() => load(), [load, refresh]);

  return { stats, error };
}
