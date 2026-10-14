import type { Page } from "@playwright/test";

/**
 * A stand-in for the Tauri backend, installed in the page before the app
 * loads. It answers the commands the interface calls, keeps a log of them
 * (`window.__MOCK__.calls`), and lets a test push events (`window.__emit`).
 */

const ev = () => ({
  notification: true,
  sound_enabled: true,
  sound: { kind: "builtin", value: "chime" },
  overlay: false,
  bring_to_front: false,
});

export const defaultSettings = {
  timer: {
    work_secs: 1500,
    short_break_secs: 300,
    long_break_secs: 900,
    sessions_before_long: 4,
    auto_start: false,
    sleep_behavior: "auto_pause",
  },
  widget_size: "medium",
  countdown_secs: 600,
  countdown_presets: [300, 600, 900, 1800],
  daily_goal: 8,
  launch_at_login: false,
  check_for_updates: true,
  shortcuts: {
    start_pause: "CommandOrControl+Alt+P",
    skip: "CommandOrControl+Alt+N",
    toggle_widget: "CommandOrControl+Alt+T",
  },
  language: "en",
  alerts: {
    work_end: ev(),
    short_break_end: ev(),
    long_break_end: ev(),
    countdown_end: ev(),
    volume: 70,
    muted: false,
  },
};

export const idleWork = {
  mode: "pomodoro",
  state: "idle",
  phase: "work",
  total_ms: 1_500_000,
  remaining_ms: 1_500_000,
  segment: null,
  label: null,
  cycle: { position: 1, length: 4 },
};

const day = (n: number) => `2026-10-${String(n).padStart(2, "0")}`;

export const weekStats = {
  period: "week",
  start: day(12),
  end: day(18),
  today: { completed: 3, focus_secs: 4500, goal: 8 },
  buckets: [12, 13, 14, 15, 16, 17, 18].map((d, i) => ({
    key: day(d),
    completed: [2, 4, 3, 0, 0, 0, 0][i],
    focus_secs: [3000, 6000, 4500, 0, 0, 0, 0][i],
  })),
  streak: { current: 3, longest: 5 },
  completion: { completed: 9, skipped: 2, interrupted: 1 },
  hours: Array.from({ length: 24 }, (_, h) => (h === 9 ? 4 : h === 14 ? 3 : 0)),
  totals: { completed: 9, focus_secs: 13500, active_days: 3, average_focus_secs: 4500 },
};

export const okStatus = {
  shortcuts: [
    { action: "start_pause", accelerator: "CommandOrControl+Alt+P", error: null },
    { action: "skip", accelerator: "CommandOrControl+Alt+N", error: null },
    { action: "toggle_widget", accelerator: "CommandOrControl+Alt+T", error: null },
  ],
  autostart_error: null,
};

/** What the update check knows before any check ran. */
export const idleUpdate = {
  current: "0.1.0",
  available: null,
  checking: false,
  last_checked: null,
  error: null,
};

export interface MockOptions {
  /** The window label: main, settings, stats or overlay. */
  label?: string;
  settings?: typeof defaultSettings;
  snapshot?: object;
  stats?: object;
  status?: object;
  /** The status the backend reports once the settings were saved. */
  statusAfterSave?: object;
  /** Makes `settings_save` fail with this message. */
  saveError?: string;
  /** What the update check reports at the start. */
  update?: object;
  /** What the update check reports once "Check now" ran. */
  updateAfterCheck?: object;
  /** Makes `update_check` fail with this message. */
  updateCheckError?: string;
  /** Query string to open the page with (the overlay reads it). */
  query?: string;
}

export interface MockCall {
  cmd: string;
  args: Record<string, unknown>;
}

/** Installs the fake backend, then opens the app. */
export async function openApp(page: Page, options: MockOptions = {}): Promise<void> {
  const config = {
    label: "main",
    settings: defaultSettings,
    snapshot: idleWork,
    stats: weekStats,
    status: okStatus,
    update: idleUpdate,
    ...options,
  };
  await page.addInitScript((opts) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const w = window as any;
    const state = {
      settings: opts.settings,
      exercises: [{ name: "Plank", amount: "30 s" }],
      snapshot: opts.snapshot,
      status: opts.status,
      update: opts.update,
      calls: [] as { cmd: string; args: unknown }[],
      listeners: {} as Record<string, { handler: number; id: number }[]>,
      callbacks: {} as Record<number, (e: unknown) => void>,
    };
    w.__MOCK__ = state;
    let next = 1;
    const emit = (event: string, payload: unknown) => {
      for (const l of state.listeners[event] ?? []) {
        state.callbacks[l.handler]?.({ event, id: l.id, payload });
      }
    };
    w.__emit = emit;
    w.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    w.__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: opts.label },
        currentWebview: { label: opts.label, windowLabel: opts.label },
      },
      transformCallback(callback: (e: unknown) => void) {
        const id = next++;
        state.callbacks[id] = callback;
        return id;
      },
      unregisterCallback(id: number) {
        delete state.callbacks[id];
      },
      convertFileSrc: (path: string) => path,
      async invoke(cmd: string, args: Record<string, unknown> = {}) {
        state.calls.push({ cmd, args });
        switch (cmd) {
          case "plugin:event|listen": {
            const event = args.event as string;
            const id = next++;
            (state.listeners[event] ??= []).push({ handler: args.handler as number, id });
            return id;
          }
          case "plugin:event|unlisten":
            return null;
          case "settings_get":
            return { settings: state.settings, exercises: state.exercises };
          case "settings_save": {
            if (opts.saveError) {
              throw opts.saveError;
            }
            state.settings = args.settings;
            state.exercises = args.exercises as typeof state.exercises;
            if (opts.statusAfterSave) {
              state.status = opts.statusAfterSave;
            }
            const payload = { settings: state.settings, exercises: state.exercises };
            emit("settings:changed", payload);
            return payload;
          }
          case "desktop_status":
            return state.status;
          case "update_status":
            return state.update;
          case "update_check": {
            if (opts.updateCheckError) {
              throw opts.updateCheckError;
            }
            if (opts.updateAfterCheck) {
              state.update = opts.updateAfterCheck;
              emit("update:changed", state.update);
            }
            return state.update;
          }
          case "engine_snapshot":
            return state.snapshot;
          case "engine_toggle": {
            const s = state.snapshot as { state: string };
            // A new object each time, as the backend sends one.
            state.snapshot = { ...s, state: s.state === "running" ? "paused" : "running" };
            emit("engine:state", state.snapshot);
            return null;
          }
          case "stats_get":
            return { ...opts.stats, period: args.period };
          default:
            return null;
        }
      },
    };
  }, config);
  await page.goto(`/${config.query ?? ""}`);
}

/** The commands the page has sent so far. */
export async function calls(page: Page): Promise<MockCall[]> {
  return page.evaluate(
    () => (window as unknown as { __MOCK__: { calls: MockCall[] } }).__MOCK__.calls,
  );
}

/** Sends an event to the page, as the backend would. */
export async function emit(page: Page, event: string, payload: unknown): Promise<void> {
  await page.evaluate(
    ([name, data]) => (window as unknown as { __emit: (e: string, p: unknown) => void }).__emit(name as string, data),
    [event, payload] as const,
  );
}
