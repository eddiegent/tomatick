import { useEffect, useRef, useState, type ReactNode } from "react";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./settings.css";
import { playSound } from "../alerts/sound";
import type { AlertEvent, Exercise, SleepBehavior, TimerSettings } from "../engine/api";
import { LANGUAGES } from "../i18n";
import { formatDuration, parseDuration } from "../engine/duration";
import {
  desktopApi,
  overlayApi,
  settingsApi,
  soundApi,
  type AlertSettings,
  type AppSettings,
  type BuiltinSound,
  type DesktopStatus,
  type EventAlerts,
  type OverlayKind,
  type SoundChoice,
  type WidgetSize,
} from "./api";
import {
  DEFAULT_SHORTCUTS,
  displayShortcut,
  isMac,
  shortcutFromKeys,
  type ShortcutAction,
  type Shortcuts,
} from "./shortcut";

const ALERT_EVENTS: { key: AlertEvent; labelKey: string; overlay?: OverlayKind }[] = [
  { key: "work_end", labelKey: "settings.alertWorkEnd", overlay: "break" },
  { key: "short_break_end", labelKey: "settings.alertShortBreakEnd" },
  { key: "long_break_end", labelKey: "settings.alertLongBreakEnd" },
  { key: "countdown_end", labelKey: "settings.alertCountdownEnd", overlay: "countdown" },
];

const BUILTIN_SOUND_LABELS: Record<BuiltinSound, string> = {
  chime: "settings.soundChime",
  bell: "settings.soundBell",
  beep: "settings.soundBeep",
  marimba: "settings.soundMarimba",
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
  goal: [1, 50],
} as const;

/** The form values, kept as text so that typing is never fought. */
interface Form {
  work: string;
  shortBreak: string;
  longBreak: string;
  sessions: string;
  goal: string;
  launch: boolean;
  language: string;
  shortcuts: Shortcuts;
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
    goal: String(settings.daily_goal),
    launch: settings.launch_at_login,
    language: settings.language,
    shortcuts: settings.shortcuts,
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
function fromForm(form: Form, t: TFunction): { settings: AppSettings } | { error: string } {
  const work = whole(form.work, LIMITS.work);
  const shortBreak = whole(form.shortBreak, LIMITS.shortBreak);
  const longBreak = whole(form.longBreak, LIMITS.longBreak);
  const sessions = whole(form.sessions, LIMITS.sessions);
  const goal = whole(form.goal, LIMITS.goal);
  const countdown = parseDuration(form.countdown);
  const presets = parsePresets(form.presets);

  if (work === null) {
    return { error: t("settings.errWork", { min: LIMITS.work[0], max: LIMITS.work[1] }) };
  }
  if (shortBreak === null) {
    return { error: t("settings.errShortBreak", { min: LIMITS.shortBreak[0], max: LIMITS.shortBreak[1] }) };
  }
  if (longBreak === null) {
    return { error: t("settings.errLongBreak", { min: LIMITS.longBreak[0], max: LIMITS.longBreak[1] }) };
  }
  if (sessions === null) {
    return { error: t("settings.errSessions", { min: LIMITS.sessions[0], max: LIMITS.sessions[1] }) };
  }
  if (goal === null) {
    return { error: t("settings.errGoal", { min: LIMITS.goal[0], max: LIMITS.goal[1] }) };
  }
  if (countdown === null) {
    return { error: t("settings.errCountdown") };
  }
  if (presets === null) {
    return { error: t("settings.errPresets", { max: MAX_PRESETS }) };
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
      daily_goal: goal,
      launch_at_login: form.launch,
      language: form.language,
      shortcuts: form.shortcuts,
      alerts: form.alerts,
    },
  };
}

const SHORTCUT_LABELS: { key: ShortcutAction; label: string }[] = [
  { key: "start_pause", label: "settings.shortcutStartPause" },
  { key: "skip", label: "settings.shortcutSkip" },
  { key: "toggle_widget", label: "settings.shortcutToggle" },
];

/** A shortcut that is changed by pressing the new keys. */
function ShortcutField({
  label,
  value,
  error,
  onChange,
  onDefault,
}: {
  label: string;
  value: string;
  error: string | null;
  onChange: (value: string) => void;
  onDefault: () => void;
}) {
  const { t } = useTranslation();
  const [recording, setRecording] = useState(false);
  const [hint, setHint] = useState<string | null>(null);
  const mac = isMac();

  const onKeyDown = (event: React.KeyboardEvent<HTMLButtonElement>) => {
    if (!recording) {
      return;
    }
    // Keep the press from reaching the window, where Escape closes it.
    event.preventDefault();
    event.nativeEvent.stopPropagation();
    if (event.key === "Escape") {
      setRecording(false);
      setHint(null);
      return;
    }
    const result = shortcutFromKeys(event.nativeEvent, mac);
    if (result === null) {
      return;
    }
    if (result === "needs-modifier") {
      setHint(t("settings.shortcutNeedsModifier"));
      return;
    }
    onChange(result);
    setRecording(false);
    setHint(null);
  };

  return (
    <div className="shortcut-row">
      <span className="shortcut-label">{label}</span>
      <button
        type="button"
        className={recording ? "shortcut recording" : "shortcut"}
        aria-label={`${label}: ${value === "" ? t("settings.shortcutNone") : displayShortcut(value, mac)}`}
        onClick={() => setRecording(true)}
        onBlur={() => {
          setRecording(false);
          setHint(null);
        }}
        onKeyDown={onKeyDown}
      >
        {recording ? t("settings.shortcutPress") : value === "" ? t("settings.shortcutNone") : displayShortcut(value, mac)}
      </button>
      <button type="button" className="secondary" onClick={onDefault}>
        {t("settings.shortcutDefault")}
      </button>
      <button type="button" className="secondary" onClick={() => onChange("")}>
        {t("settings.shortcutClear")}
      </button>
      {(hint || error) && (
        <p className="error shortcut-error">{hint ?? `${t("settings.shortcutFailed")}${error}`}</p>
      )}
    </div>
  );
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
  const { t } = useTranslation();
  const [form, setForm] = useState<Form | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [desktop, setDesktop] = useState<DesktopStatus | null>(null);
  // The exercise being dragged and the row it would land on.
  const [drag, setDrag] = useState<{ from: number; over: number } | null>(null);
  const dragging = useRef(false);
  const listRef = useRef<HTMLOListElement>(null);

  useEffect(() => {
    settingsApi
      .get()
      .then(({ settings, exercises }) => setForm(toForm(settings, exercises)))
      .catch((e) => setError(String(e)));
    desktopApi
      .status()
      .then(setDesktop)
      .catch(() => {});
  }, []);

  useEffect(() => {
    document.title = t("settings.title");
  }, [t]);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        // Escape cancels a drag in progress before it closes the window.
        if (dragging.current) {
          dragging.current = false;
          setDrag(null);
          return;
        }
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

  const moveExerciseTo = (from: number, to: number) => {
    if (from === to) {
      return;
    }
    const next = [...form.exercises];
    const [item] = next.splice(from, 1);
    next.splice(to, 0, item);
    change({ exercises: next });
  };

  /** The row under the pointer, the first or last one when it is outside. */
  const rowAt = (clientY: number): number => {
    const rows = listRef.current?.children;
    if (!rows || rows.length === 0) {
      return 0;
    }
    for (let i = 0; i < rows.length; i += 1) {
      if (clientY < rows[i].getBoundingClientRect().bottom) {
        return i;
      }
    }
    return rows.length - 1;
  };

  const editEvent = (key: AlertEvent, patch: Partial<EventAlerts>) =>
    change({
      alerts: { ...form.alerts, [key]: { ...form.alerts[key], ...patch } },
    });

  const editAlerts = (patch: Partial<Pick<AlertSettings, "volume" | "muted">>) =>
    change({ alerts: { ...form.alerts, ...patch } });

  const testSound = (key: AlertEvent) => {
    playSound(form.alerts[key].sound, form.alerts.volume).catch((e) =>
      setError(`${t("settings.soundFailed")}${String(e)}`),
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
      setNotice(t("settings.resetDone"));
    } catch (e) {
      setError(String(e));
    }
  };

  const save = async () => {
    const result = fromForm(form, t);
    if ("error" in result) {
      setError(result.error);
      return;
    }
    setSaving(true);
    try {
      await settingsApi.save(result.settings, form.exercises);
      // A shortcut another application holds, or a refused login item, does
      // not undo the save. The window stays open so the user can see why.
      const status = await desktopApi.status().catch(() => null);
      setDesktop(status);
      const failed =
        status !== null &&
        (status.autostart_error !== null || status.shortcuts.some((s) => s.error !== null));
      if (failed) {
        setError(t("settings.desktopProblems"));
        setSaving(false);
        return;
      }
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
          <h2>{t("settings.cycle")}</h2>
          <div className="grid">
            <Field label={t("settings.work")}>
              <input
                type="number"
                min={LIMITS.work[0]}
                max={LIMITS.work[1]}
                value={form.work}
                onChange={(e) => change({ work: e.target.value })}
              />
            </Field>
            <Field label={t("settings.shortBreak")}>
              <input
                type="number"
                min={LIMITS.shortBreak[0]}
                max={LIMITS.shortBreak[1]}
                value={form.shortBreak}
                onChange={(e) => change({ shortBreak: e.target.value })}
              />
            </Field>
            <Field label={t("settings.longBreak")}>
              <input
                type="number"
                min={LIMITS.longBreak[0]}
                max={LIMITS.longBreak[1]}
                value={form.longBreak}
                onChange={(e) => change({ longBreak: e.target.value })}
              />
            </Field>
            <Field label={t("settings.sessions")}>
              <input
                type="number"
                min={LIMITS.sessions[0]}
                max={LIMITS.sessions[1]}
                value={form.sessions}
                onChange={(e) => change({ sessions: e.target.value })}
              />
            </Field>
            <Field label={t("settings.goal")}>
              <input
                type="number"
                min={LIMITS.goal[0]}
                max={LIMITS.goal[1]}
                value={form.goal}
                onChange={(e) => change({ goal: e.target.value })}
              />
            </Field>
          </div>
          <label className="check">
            <input
              type="checkbox"
              checked={form.autoStart}
              onChange={(e) => change({ autoStart: e.target.checked })}
            />
            <span>{t("settings.autoStart")}</span>
          </label>
          <Field label={t("settings.sleep")}>
            <select
              value={form.sleep}
              onChange={(e) => change({ sleep: e.target.value as SleepBehavior })}
            >
              <option value="auto_pause">{t("settings.sleepPause")}</option>
              <option value="keep_running">{t("settings.sleepKeep")}</option>
            </select>
          </Field>
        </section>

        <section>
          <h2>{t("settings.widget")}</h2>
          <div className="segmented" role="radiogroup" aria-label={t("settings.widget")}>
            {(["small", "medium", "large"] as const).map((size) => (
              <label key={size} className={form.size === size ? "selected" : ""}>
                <input
                  type="radio"
                  name="size"
                  checked={form.size === size}
                  onChange={() => change({ size })}
                />
                {t(`settings.${size}`)}
              </label>
            ))}
          </div>
        </section>

        <section>
          <h2>{t("settings.system")}</h2>
          <label className="check">
            <input
              type="checkbox"
              checked={form.launch}
              onChange={(e) => change({ launch: e.target.checked })}
            />
            {t("settings.launch")}
          </label>
          {desktop?.autostart_error && (
            <p className="error">{`${t("settings.launchFailed")}${desktop.autostart_error}`}</p>
          )}
          <Field label={t("settings.language")}>
            <select value={form.language} onChange={(e) => change({ language: e.target.value })}>
              <option value="system">{t("settings.languageSystem")}</option>
              {LANGUAGES.map(({ code, name }) => (
                <option key={code} value={code}>
                  {name}
                </option>
              ))}
            </select>
          </Field>
          <h3>{t("settings.shortcuts")}</h3>
          <p className="help">{t("settings.shortcutsHelp")}</p>
          {SHORTCUT_LABELS.map(({ key, label }) => (
            <ShortcutField
              key={key}
              label={t(label)}
              value={form.shortcuts[key]}
              error={desktop?.shortcuts.find((s) => s.action === key)?.error ?? null}
              onChange={(value) => change({ shortcuts: { ...form.shortcuts, [key]: value } })}
              onDefault={() =>
                change({ shortcuts: { ...form.shortcuts, [key]: DEFAULT_SHORTCUTS[key] } })
              }
            />
          ))}
        </section>

        <section>
          <h2>{t("settings.countdown")}</h2>
          <Field label={t("settings.countdownStart")}>
            <input
              type="text"
              inputMode="numeric"
              value={form.countdown}
              onChange={(e) => change({ countdown: e.target.value })}
            />
          </Field>
          <p className="help">{t("settings.countdownHelp")}</p>
          <Field label={t("settings.countdownPresets")}>
            <input
              type="text"
              value={form.presets}
              onChange={(e) => change({ presets: e.target.value })}
            />
          </Field>
          <p className="help">{t("settings.countdownPresetsHelp")}</p>
        </section>

        <section>
          <h2>{t("settings.alerts")}</h2>
          <p className="help">{t("settings.alertsHelp")}</p>
          <label className="check">
            <input
              type="checkbox"
              checked={form.alerts.muted}
              onChange={(e) => editAlerts({ muted: e.target.checked })}
            />
            <span>{t("settings.mute")}</span>
          </label>
          <label className="field volume">
            <span>
              {t("settings.volume")}: {form.alerts.volume}%
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
          {ALERT_EVENTS.map(({ key, labelKey, overlay }) => {
            const label = t(labelKey);
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
                  <span>{t("settings.notification")}</span>
                </label>
                <label className="check">
                  <input
                    type="checkbox"
                    checked={alerts.bring_to_front}
                    onChange={(e) => editEvent(key, { bring_to_front: e.target.checked })}
                  />
                  <span>{t("settings.bringToFront")}</span>
                </label>
                {overlay && (
                  <div className="overlay-row">
                    <label className="check">
                      <input
                        type="checkbox"
                        checked={alerts.overlay}
                        onChange={(e) => editEvent(key, { overlay: e.target.checked })}
                      />
                      <span>{t("settings.overlay")}</span>
                    </label>
                    <button
                      type="button"
                      title={t("settings.previewOverlayHint")}
                      onClick={() => overlayApi.preview(overlay).catch(() => undefined)}
                    >
                      {t("settings.previewOverlay")}
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
                    <span>{t("settings.sound")}</span>
                  </label>
                  <select
                    aria-label={`${label}: ${t("settings.sound")}`}
                    disabled={!alerts.sound_enabled}
                    value={soundKey(alerts.sound)}
                    onChange={(e) => editEvent(key, { sound: soundFromKey(e.target.value) })}
                  >
                    <optgroup label={t("settings.builtinSounds")}>
                      {(Object.keys(BUILTIN_SOUND_LABELS) as BuiltinSound[]).map((id) => (
                        <option key={id} value={`builtin:${id}`}>
                          {t(BUILTIN_SOUND_LABELS[id])}
                        </option>
                      ))}
                    </optgroup>
                    {customSounds.length > 0 && (
                      <optgroup label={t("settings.yourSounds")}>
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
                    title={form.alerts.muted ? t("settings.testMuted") : t("settings.testHint")}
                    onClick={() => testSound(key)}
                  >
                    {t("settings.test")}
                  </button>
                  <button
                    type="button"
                    title={t("settings.addFileHint")}
                    onClick={() => addSound(key)}
                  >
                    {t("settings.addFile")}
                  </button>
                </div>
              </fieldset>
            );
          })}
        </section>

        <section>
          <h2>{t("settings.exercises")}</h2>
          <p className="help">{t("settings.exercisesHelp")}</p>
          <ol ref={listRef} className={drag ? "exercises sorting" : "exercises"}>
            {form.exercises.map((exercise, index) => (
              <li
                key={index}
                className={
                  drag && index === drag.from
                    ? "dragging"
                    : drag && index === drag.over
                      ? drag.over > drag.from
                        ? "drop-after"
                        : "drop-before"
                      : undefined
                }
              >
                <button
                  type="button"
                  className="icon handle"
                  title={t("settings.dragHandle")}
                  aria-label={t("settings.dragHandle")}
                  onPointerDown={(event) => {
                    if (event.button !== 0) {
                      return;
                    }
                    event.currentTarget.setPointerCapture(event.pointerId);
                    dragging.current = true;
                    setDrag({ from: index, over: index });
                  }}
                  onPointerMove={(event) => {
                    if (!dragging.current) {
                      return;
                    }
                    const over = rowAt(event.clientY);
                    setDrag((current) =>
                      current && current.over !== over ? { ...current, over } : current,
                    );
                  }}
                  onPointerUp={(event) => {
                    if (!dragging.current) {
                      return;
                    }
                    dragging.current = false;
                    setDrag(null);
                    moveExerciseTo(index, rowAt(event.clientY));
                  }}
                  onPointerCancel={() => {
                    dragging.current = false;
                    setDrag(null);
                  }}
                >
                  {"≡"}
                </button>
                <input
                  type="text"
                  placeholder={t("settings.exerciseName")}
                  aria-label={t("settings.exerciseName")}
                  maxLength={60}
                  value={exercise.name}
                  onChange={(e) => editExercise(index, { name: e.target.value })}
                />
                <input
                  type="text"
                  className="amount"
                  placeholder={t("settings.exerciseAmount")}
                  aria-label={t("settings.exerciseAmount")}
                  maxLength={60}
                  value={exercise.amount ?? ""}
                  onChange={(e) => editExercise(index, { amount: e.target.value })}
                />
                <button
                  type="button"
                  className="icon"
                  title={t("settings.moveUp")}
                  aria-label={t("settings.moveUp")}
                  disabled={index === 0}
                  onClick={() => moveExercise(index, -1)}
                >
                  {"↑"}
                </button>
                <button
                  type="button"
                  className="icon"
                  title={t("settings.moveDown")}
                  aria-label={t("settings.moveDown")}
                  disabled={index === form.exercises.length - 1}
                  onClick={() => moveExercise(index, 1)}
                >
                  {"↓"}
                </button>
                <button
                  type="button"
                  className="icon"
                  title={t("settings.remove")}
                  aria-label={t("settings.remove")}
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
            {t("settings.add")}
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
            {t("settings.reset")}
          </button>
          <button type="button" className="secondary" onClick={closeWindow}>
            {t("settings.cancel")}
          </button>
          <button type="button" className="primary" disabled={saving} onClick={save}>
            {saving ? t("settings.saving") : t("settings.save")}
          </button>
        </div>
      </footer>
    </div>
  );
}
