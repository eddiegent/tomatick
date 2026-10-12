import { useEffect, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./settings.css";
import type { Exercise, SleepBehavior, TimerSettings } from "../engine/api";
import { formatDuration, parseDuration } from "../engine/duration";
import { settingsApi, type AppSettings, type WidgetSize } from "./api";

// English only for now. The text moves to the translation files in
// milestone 10.
const TEXT = {
  title: "Settings",
  cycle: "Pomodoro cycle",
  work: "Work (minutes)",
  shortBreak: "Short break (minutes)",
  longBreak: "Long break (minutes)",
  sessions: "Work sessions before a long break",
  autoStart: "Start the next phase automatically",
  sleep: "When the computer sleeps or locks",
  sleepPause: "Pause the timer",
  sleepKeep: "Keep the timer running",
  widget: "Widget",
  small: "Small",
  medium: "Medium",
  large: "Large",
  countdown: "Countdown timer",
  countdownStart: "Starting value",
  countdownHelp: "Minutes, or m:ss (for example 10 or 1:30).",
  exercises: "Break exercises",
  exercisesHelp:
    "The app goes through this list in order, one entry per short break. A long break is split into several short ones and uses the list the same way.",
  exerciseName: "Exercise",
  exerciseAmount: "Amount (optional)",
  moveUp: "Move up",
  moveDown: "Move down",
  remove: "Remove",
  add: "Add an exercise",
  reset: "Reset to classic",
  resetDone: "Classic values loaded (25/5/15, 4 sessions). Save to apply them.",
  cancel: "Cancel",
  save: "Save",
  saving: "Saving",
};

const LIMITS = {
  work: [1, 180],
  shortBreak: [1, 60],
  longBreak: [1, 120],
  sessions: [2, 12],
} as const;

/** The form values, kept as text so that typing is never fought. */
interface Form {
  work: string;
  shortBreak: string;
  longBreak: string;
  sessions: string;
  autoStart: boolean;
  sleep: SleepBehavior;
  size: WidgetSize;
  countdown: string;
  exercises: Exercise[];
}

function toForm(settings: AppSettings, exercises: Exercise[]): Form {
  return {
    ...timerToForm(settings.timer),
    size: settings.widget_size,
    countdown: formatDuration(settings.countdown_secs),
    exercises,
  };
}

function timerToForm(timer: TimerSettings) {
  return {
    work: String(timer.work_secs / 60),
    shortBreak: String(timer.short_break_secs / 60),
    longBreak: String(timer.long_break_secs / 60),
    sessions: String(timer.sessions_before_long),
    autoStart: timer.auto_start,
    sleep: timer.sleep_behavior,
  };
}

/** Reads a whole number in a range, or `null`. */
function whole(text: string, [min, max]: readonly [number, number]): number | null {
  if (!/^\d+$/.test(text.trim())) {
    return null;
  }
  const value = Number(text);
  return value >= min && value <= max ? value : null;
}

/** Turns the form into settings, or returns the first problem found. */
function fromForm(form: Form): { settings: AppSettings } | { error: string } {
  const work = whole(form.work, LIMITS.work);
  const shortBreak = whole(form.shortBreak, LIMITS.shortBreak);
  const longBreak = whole(form.longBreak, LIMITS.longBreak);
  const sessions = whole(form.sessions, LIMITS.sessions);
  const countdown = parseDuration(form.countdown);

  if (work === null) {
    return { error: `Work must be a whole number from ${LIMITS.work[0]} to ${LIMITS.work[1]} minutes.` };
  }
  if (shortBreak === null) {
    return { error: `Short break must be a whole number from ${LIMITS.shortBreak[0]} to ${LIMITS.shortBreak[1]} minutes.` };
  }
  if (longBreak === null) {
    return { error: `Long break must be a whole number from ${LIMITS.longBreak[0]} to ${LIMITS.longBreak[1]} minutes.` };
  }
  if (sessions === null) {
    return { error: `Sessions must be a whole number from ${LIMITS.sessions[0]} to ${LIMITS.sessions[1]}.` };
  }
  if (countdown === null) {
    return { error: "The countdown must be between 0:01 and 23:59:59." };
  }
  return {
    settings: {
      timer: {
        work_secs: work * 60,
        short_break_secs: shortBreak * 60,
        long_break_secs: longBreak * 60,
        sessions_before_long: sessions,
        auto_start: form.autoStart,
        sleep_behavior: form.sleep,
      },
      widget_size: form.size,
      countdown_secs: countdown,
    },
  };
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="field">
      <span>{label}</span>
      {children}
    </label>
  );
}

function closeWindow() {
  getCurrentWindow()
    .close()
    .catch(() => {});
}

/** The settings window, opened from the cog of the widget. */
export function SettingsDialog() {
  const [form, setForm] = useState<Form | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    document.title = TEXT.title;
    settingsApi
      .get()
      .then(({ settings, exercises }) => setForm(toForm(settings, exercises)))
      .catch((e) => setError(String(e)));
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeWindow();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  if (!form) {
    return error ? <p className="error page-error">{error}</p> : null;
  }

  const change = (patch: Partial<Form>) => {
    setForm({ ...form, ...patch });
    setError(null);
    setNotice(null);
  };

  const editExercise = (index: number, patch: Partial<Exercise>) =>
    change({
      exercises: form.exercises.map((item, i) => (i === index ? { ...item, ...patch } : item)),
    });

  const moveExercise = (index: number, offset: number) => {
    const target = index + offset;
    if (target < 0 || target >= form.exercises.length) {
      return;
    }
    const next = [...form.exercises];
    [next[index], next[target]] = [next[target], next[index]];
    change({ exercises: next });
  };

  const resetToClassic = async () => {
    try {
      const classic = await settingsApi.classic();
      setForm({ ...form, ...timerToForm(classic) });
      setError(null);
      setNotice(TEXT.resetDone);
    } catch (e) {
      setError(String(e));
    }
  };

  const save = async () => {
    const result = fromForm(form);
    if ("error" in result) {
      setError(result.error);
      return;
    }
    setSaving(true);
    try {
      await settingsApi.save(result.settings, form.exercises);
      closeWindow();
    } catch (e) {
      setError(String(e));
      setSaving(false);
    }
  };

  return (
    <div className="settings">
      <main>
        <section>
          <h2>{TEXT.cycle}</h2>
          <div className="grid">
            <Field label={TEXT.work}>
              <input
                type="number"
                min={LIMITS.work[0]}
                max={LIMITS.work[1]}
                value={form.work}
                onChange={(e) => change({ work: e.target.value })}
              />
            </Field>
            <Field label={TEXT.shortBreak}>
              <input
                type="number"
                min={LIMITS.shortBreak[0]}
                max={LIMITS.shortBreak[1]}
                value={form.shortBreak}
                onChange={(e) => change({ shortBreak: e.target.value })}
              />
            </Field>
            <Field label={TEXT.longBreak}>
              <input
                type="number"
                min={LIMITS.longBreak[0]}
                max={LIMITS.longBreak[1]}
                value={form.longBreak}
                onChange={(e) => change({ longBreak: e.target.value })}
              />
            </Field>
            <Field label={TEXT.sessions}>
              <input
                type="number"
                min={LIMITS.sessions[0]}
                max={LIMITS.sessions[1]}
                value={form.sessions}
                onChange={(e) => change({ sessions: e.target.value })}
              />
            </Field>
          </div>
          <label className="check">
            <input
              type="checkbox"
              checked={form.autoStart}
              onChange={(e) => change({ autoStart: e.target.checked })}
            />
            <span>{TEXT.autoStart}</span>
          </label>
          <Field label={TEXT.sleep}>
            <select
              value={form.sleep}
              onChange={(e) => change({ sleep: e.target.value as SleepBehavior })}
            >
              <option value="auto_pause">{TEXT.sleepPause}</option>
              <option value="keep_running">{TEXT.sleepKeep}</option>
            </select>
          </Field>
        </section>

        <section>
          <h2>{TEXT.widget}</h2>
          <div className="segmented" role="radiogroup" aria-label={TEXT.widget}>
            {(["small", "medium", "large"] as const).map((size) => (
              <label key={size} className={form.size === size ? "selected" : ""}>
                <input
                  type="radio"
                  name="size"
                  checked={form.size === size}
                  onChange={() => change({ size })}
                />
                {TEXT[size]}
              </label>
            ))}
          </div>
        </section>

        <section>
          <h2>{TEXT.countdown}</h2>
          <Field label={TEXT.countdownStart}>
            <input
              type="text"
              inputMode="numeric"
              value={form.countdown}
              onChange={(e) => change({ countdown: e.target.value })}
            />
          </Field>
          <p className="help">{TEXT.countdownHelp}</p>
        </section>

        <section>
          <h2>{TEXT.exercises}</h2>
          <p className="help">{TEXT.exercisesHelp}</p>
          <ol className="exercises">
            {form.exercises.map((exercise, index) => (
              <li key={index}>
                <input
                  type="text"
                  placeholder={TEXT.exerciseName}
                  aria-label={TEXT.exerciseName}
                  maxLength={60}
                  value={exercise.name}
                  onChange={(e) => editExercise(index, { name: e.target.value })}
                />
                <input
                  type="text"
                  className="amount"
                  placeholder={TEXT.exerciseAmount}
                  aria-label={TEXT.exerciseAmount}
                  maxLength={60}
                  value={exercise.amount ?? ""}
                  onChange={(e) => editExercise(index, { amount: e.target.value })}
                />
                <button
                  type="button"
                  className="icon"
                  title={TEXT.moveUp}
                  aria-label={TEXT.moveUp}
                  disabled={index === 0}
                  onClick={() => moveExercise(index, -1)}
                >
                  {"↑"}
                </button>
                <button
                  type="button"
                  className="icon"
                  title={TEXT.moveDown}
                  aria-label={TEXT.moveDown}
                  disabled={index === form.exercises.length - 1}
                  onClick={() => moveExercise(index, 1)}
                >
                  {"↓"}
                </button>
                <button
                  type="button"
                  className="icon"
                  title={TEXT.remove}
                  aria-label={TEXT.remove}
                  onClick={() =>
                    change({ exercises: form.exercises.filter((_, i) => i !== index) })
                  }
                >
                  {"×"}
                </button>
              </li>
            ))}
          </ol>
          <button
            type="button"
            className="secondary"
            disabled={form.exercises.length >= 50}
            onClick={() => change({ exercises: [...form.exercises, { name: "", amount: null }] })}
          >
            {TEXT.add}
          </button>
        </section>
      </main>

      <footer>
        <div className="messages" aria-live="polite">
          {error && <p className="error">{error}</p>}
          {!error && notice && <p className="notice">{notice}</p>}
        </div>
        <div className="buttons">
          <button type="button" className="secondary left" onClick={resetToClassic}>
            {TEXT.reset}
          </button>
          <button type="button" className="secondary" onClick={closeWindow}>
            {TEXT.cancel}
          </button>
          <button type="button" className="primary" disabled={saving} onClick={save}>
            {saving ? TEXT.saving : TEXT.save}
          </button>
        </div>
      </footer>
    </div>
  );
}
