import { useEffect, useState, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./settings.css";
import { playSound } from "../alerts/sound";
import type { AlertEvent, Exercise, SleepBehavior, TimerSettings } from "../engine/api";
import { formatDuration, parseDuration } from "../engine/duration";
import {
  overlayApi,
  settingsApi,
  soundApi,
  type AlertSettings,
  type AppSettings,
  type BuiltinSound,
  type EventAlerts,
  type OverlayKind,
  type SoundChoice,
  type WidgetSize,
} from "./api";

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
  countdownPresets: "Quick choices",
  countdownPresetsHelp:
    "One to six durations, separated by commas (for example 5, 10, 15, 30 or 1:30). They appear when you edit the countdown on the widget.",
  alerts: "Alerts",
  alertsHelp:
    "What happens when a phase ends. Sounds play through the app, independent of the system volume.",
  volume: "Volume",
  mute: "Mute all alert sounds",
  notification: "System notification",
  bringToFront: "Bring the widget to the front",
  overlay: "Full-screen overlay",
  previewOverlay: "Preview overlay",
  previewOverlayHint: "Show the overlay with sample content. The timer is not touched.",
  sound: "Sound",
  builtinSounds: "Built-in",
  yourSounds: "Your files",
  test: "Test",
  testHint: "Play this sound",
  testMuted: "Unmute to test",
  addFile: "Add a file",
  addFileHint: "Use your own MP3, WAV or OGG file",
  soundFailed: "This sound cannot be played: ",
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

const ALERT_EVENTS: { key: AlertEvent; label: string; overlay?: OverlayKind }[] = [
  { key: "work_end", label: "Work ends (a break starts)", overlay: "break" },
  { key: "short_break_end", label: "Short break ends" },
  { key: "long_break_end", label: "Long break ends" },
  { key: "countdown_end", label: "Countdown ends", overlay: "countdown" },
];

const BUILTIN_SOUND_LABELS: Record<BuiltinSound, string> = {
  chime: "Soft chime",
  bell: "Bell",
  beep: "Gentle beep",
  marimba: "Marimba",
};

const soundKey = (choice: SoundChoice) => `${choice.kind}:${choice.value}`;

function soundFromKey(key: string): SoundChoice {
  const split = key.indexOf(":");
  const value = key.slice(split + 1);
  return key.slice(0, split) === "custom"
    ? { kind: "custom", value }
    : { kind: "builtin", value: value as BuiltinSound };
}

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
  presets: string;
  alerts: AlertSettings;
  exercises: Exercise[];
}

function toForm(settings: AppSettings, exercises: Exercise[]): Form {
  return {
    ...timerToForm(settings.timer),
    size: settings.widget_size,
    countdown: formatDuration(settings.countdown_secs),
    presets: settings.countdown_presets.map(formatDuration).join(", "),
    alerts: settings.alerts,
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

const MAX_PRESETS = 6;

/** Reads "5, 10, 1:30" into sorted, distinct seconds, or `null`. */
function parsePresets(text: string): number[] | null {
  const parts = text
    .split(",")
    .map((part) => part.trim())
    .filter((part) => part !== "");
  const seconds: number[] = [];
  for (const part of parts) {
    const value = parseDuration(part);
    if (value === null) {
      return null;
    }
    if (!seconds.includes(value)) {
      seconds.push(value);
    }
  }
  if (seconds.length < 1 || seconds.length > MAX_PRESETS) {
    return null;
  }
  return seconds.sort((a, b) => a - b);
}

/** Turns the form into settings, or returns the first problem found. */
function fromForm(form: Form): { settings: AppSettings } | { error: string } {
  const work = whole(form.work, LIMITS.work);
  const shortBreak = whole(form.shortBreak, LIMITS.shortBreak);
  const longBreak = whole(form.longBreak, LIMITS.longBreak);
  const sessions = whole(form.sessions, LIMITS.sessions);
  const countdown = parseDuration(form.countdown);
  const presets = parsePresets(form.presets);

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
  if (presets === null) {
    return {
      error: `Quick choices must be one to ${MAX_PRESETS} durations between 0:01 and 23:59:59, separated by commas.`,
    };
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
      countdown_presets: presets,
      alerts: form.alerts,
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

  const editEvent = (key: AlertEvent, patch: Partial<EventAlerts>) =>
    change({
      alerts: { ...form.alerts, [key]: { ...form.alerts[key], ...patch } },
    });

  const editAlerts = (patch: Partial<Pick<AlertSettings, "volume" | "muted">>) =>
    change({ alerts: { ...form.alerts, ...patch } });

  const testSound = (key: AlertEvent) => {
    playSound(form.alerts[key].sound, form.alerts.volume).catch((e) =>
      setError(`${TEXT.soundFailed}${String(e)}`),
    );
  };

  const addSound = async (key: AlertEvent) => {
    try {
      const file = await soundApi.add();
      if (file) {
        editEvent(key, { sound: { kind: "custom", value: file }, sound_enabled: true });
      }
    } catch (e) {
      setError(String(e));
    }
  };

  // The custom files that an event uses. They are offered to every event.
  const customSounds = [
    ...new Set(
      ALERT_EVENTS.flatMap(({ key }) => {
        const { sound } = form.alerts[key];
        return sound.kind === "custom" ? [sound.value] : [];
      }),
    ),
  ];

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
          <Field label={TEXT.countdownPresets}>
            <input
              type="text"
              value={form.presets}
              onChange={(e) => change({ presets: e.target.value })}
            />
          </Field>
          <p className="help">{TEXT.countdownPresetsHelp}</p>
        </section>

        <section>
          <h2>{TEXT.alerts}</h2>
          <p className="help">{TEXT.alertsHelp}</p>
          <label className="check">
            <input
              type="checkbox"
              checked={form.alerts.muted}
              onChange={(e) => editAlerts({ muted: e.target.checked })}
            />
            <span>{TEXT.mute}</span>
          </label>
          <label className="field volume">
            <span>
              {TEXT.volume}: {form.alerts.volume}%
            </span>
            <input
              type="range"
              min={0}
              max={100}
              step={5}
              value={form.alerts.volume}
              onChange={(e) => editAlerts({ volume: Number(e.target.value) })}
            />
          </label>
          {ALERT_EVENTS.map(({ key, label, overlay }) => {
            const alerts = form.alerts[key];
            return (
              <fieldset className="alert-event" key={key}>
                <legend>{label}</legend>
                <label className="check">
                  <input
                    type="checkbox"
                    checked={alerts.notification}
                    onChange={(e) => editEvent(key, { notification: e.target.checked })}
                  />
                  <span>{TEXT.notification}</span>
                </label>
                <label className="check">
                  <input
                    type="checkbox"
                    checked={alerts.bring_to_front}
                    onChange={(e) => editEvent(key, { bring_to_front: e.target.checked })}
                  />
                  <span>{TEXT.bringToFront}</span>
                </label>
                {overlay && (
                  <div className="overlay-row">
                    <label className="check">
                      <input
                        type="checkbox"
                        checked={alerts.overlay}
                        onChange={(e) => editEvent(key, { overlay: e.target.checked })}
                      />
                      <span>{TEXT.overlay}</span>
                    </label>
                    <button
                      type="button"
                      title={TEXT.previewOverlayHint}
                      onClick={() => overlayApi.preview(overlay).catch(() => undefined)}
                    >
                      {TEXT.previewOverlay}
                    </button>
                  </div>
                )}
                <div className="sound-row">
                  <label className="check">
                    <input
                      type="checkbox"
                      checked={alerts.sound_enabled}
                      onChange={(e) => editEvent(key, { sound_enabled: e.target.checked })}
                    />
                    <span>{TEXT.sound}</span>
                  </label>
                  <select
                    aria-label={`${label}: ${TEXT.sound}`}
                    disabled={!alerts.sound_enabled}
                    value={soundKey(alerts.sound)}
                    onChange={(e) => editEvent(key, { sound: soundFromKey(e.target.value) })}
                  >
                    <optgroup label={TEXT.builtinSounds}>
                      {(Object.keys(BUILTIN_SOUND_LABELS) as BuiltinSound[]).map((id) => (
                        <option key={id} value={`builtin:${id}`}>
                          {BUILTIN_SOUND_LABELS[id]}
                        </option>
                      ))}
                    </optgroup>
                    {customSounds.length > 0 && (
                      <optgroup label={TEXT.yourSounds}>
                        {customSounds.map((file) => (
                          <option key={file} value={`custom:${file}`}>
                            {file}
                          </option>
                        ))}
                      </optgroup>
                    )}
                  </select>
                  <button
                    type="button"
                    disabled={form.alerts.muted || !alerts.sound_enabled}
                    title={form.alerts.muted ? TEXT.testMuted : TEXT.testHint}
                    onClick={() => testSound(key)}
                  >
                    {TEXT.test}
                  </button>
                  <button
                    type="button"
                    title={TEXT.addFileHint}
                    onClick={() => addSound(key)}
                  >
                    {TEXT.addFile}
                  </button>
                </div>
              </fieldset>
            );
          })}
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
