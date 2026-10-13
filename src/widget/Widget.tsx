import { useEffect, useRef, useState, type CSSProperties } from "react";
import "./widget.css";
import { useAlertSounds } from "../alerts/useAlertSounds";
import { engine, formatClock, useSnapshot, type Snapshot } from "../engine/api";
import { formatDuration, parseDuration } from "../engine/duration";
import { settingsApi, useSettings } from "../settings/api";
import {
  ChartIcon,
  CogIcon,
  MoonIcon,
  PauseIcon,
  PlayIcon,
  ResetIcon,
  SkipIcon,
  SwapIcon,
} from "./icons";
import { WIDGET_PIXELS, useApplyWidgetSize } from "./size";
import { Tomato } from "./Tomato";
import { progressOf, themeOf, type Theme } from "./theme";

// English only for now. The text moves to the translation files in
// milestone 10.
const TEXT = {
  phase: {
    work: "Work",
    short_break: "Short break",
    long_break: "Long break",
    countdown: "Countdown",
  } satisfies Record<Theme, string>,
  start: "Start",
  pause: "Pause",
  resume: "Resume",
  skip: "Skip to the next phase",
  reset: "Reset",
  toCountdown: "Switch to countdown",
  toPomodoro: "Switch to Pomodoro",
  stats: "Statistics (coming soon)",
  settings: "Settings",
  setCountdown: "Click to set the countdown",
  countdownInput: "Countdown duration",
  countdownHint: "Minutes, or m:ss. Enter to set, Escape to cancel.",
  paused: "Paused",
  autoPaused: "Paused by the system",
};

/** Quick choices for the countdown, in seconds. */
const COUNTDOWN_PRESETS = [5 * 60, 10 * 60, 15 * 60, 30 * 60];

function playLabel(snapshot: Snapshot): string {
  switch (snapshot.state) {
    case "running":
      return TEXT.pause;
    case "idle":
      return TEXT.start;
    default:
      return TEXT.resume;
  }
}

/** The always-on-top tomato. */
export function Widget() {
  const snapshot = useSnapshot();
  const settings = useSettings();
  const size = settings?.settings.widget_size ?? "medium";
  useApplyWidgetSize(size);
  useAlertSounds(settings?.settings.alerts);

  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState("");
  const [invalid, setInvalid] = useState(false);
  const input = useRef<HTMLInputElement>(null);

  // The duration can only be edited while the countdown is idle.
  const editable = snapshot?.mode === "countdown" && snapshot.state === "idle";
  useEffect(() => {
    if (!editable) {
      setEditing(false);
    }
  }, [editable]);
  useEffect(() => {
    if (editing) {
      input.current?.focus();
      input.current?.select();
    }
  }, [editing]);

  const beginEdit = () => {
    if (!snapshot) {
      return;
    }
    setDraft(formatDuration(Math.round(snapshot.total_ms / 1000)));
    setInvalid(false);
    setEditing(true);
  };

  const applyCountdown = (seconds: number | null) => {
    if (seconds === null) {
      setInvalid(true);
      return;
    }
    engine
      .setCountdown(seconds)
      .then(() => setEditing(false))
      .catch(() => setInvalid(true));
  };

  // The browser context menu does not belong on a desktop widget.
  useEffect(() => {
    const block = (event: Event) => event.preventDefault();
    window.addEventListener("contextmenu", block);
    return () => window.removeEventListener("contextmenu", block);
  }, []);

  if (!snapshot || !settings) {
    return null;
  }

  const theme = themeOf(snapshot);
  const time = formatClock(snapshot.remaining_ms);
  const running = snapshot.state === "running";
  const paused = snapshot.state === "paused" || snapshot.state === "auto_paused";
  const countdown = snapshot.mode === "countdown";
  const tooltip = [
    TEXT.phase[theme],
    time,
    snapshot.label,
    paused ? (snapshot.state === "auto_paused" ? TEXT.autoPaused : TEXT.paused) : null,
  ]
    .filter(Boolean)
    .join(" | ");

  return (
    <div
      className={editing ? "widget editing" : "widget"}
      data-theme={theme}
      data-state={snapshot.state}
      data-size={size}
      style={{ "--size": `${WIDGET_PIXELS[size]}px` } as CSSProperties}
    >
      <div className="body" data-tauri-drag-region title={tooltip}>
        <Tomato progress={progressOf(snapshot)} />
        {!countdown && (
          <span className="cycle">
            {snapshot.cycle.position}/{snapshot.cycle.length}
          </span>
        )}
        {!editing && (
          <span className={time.length > 5 ? "time long" : "time"}>{time}</span>
        )}
        {snapshot.label && <span className="label">{snapshot.label}</span>}
        {paused && (
          <span className="paused-mark">
            {snapshot.state === "auto_paused" ? <MoonIcon /> : <PauseIcon />}
          </span>
        )}
      </div>

      {editable && !editing && (
        <button
          type="button"
          className="time-edit"
          aria-label={TEXT.setCountdown}
          title={TEXT.setCountdown}
          onClick={beginEdit}
        />
      )}
      {editing && (
        <>
          <input
            ref={input}
            className={invalid ? "time-input invalid" : "time-input"}
            value={draft}
            inputMode="numeric"
            maxLength={8}
            aria-label={TEXT.countdownInput}
            aria-invalid={invalid}
            title={TEXT.countdownHint}
            onChange={(event) => {
              // Pasted text can hold anything, so keep digits and colons only.
              setDraft(event.target.value.replace(/[^\d:]/g, ""));
              setInvalid(false);
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                applyCountdown(parseDuration(draft));
              } else if (event.key === "Escape") {
                setEditing(false);
              } else if (
                event.key.length === 1 &&
                !/[\d:]/.test(event.key) &&
                !event.ctrlKey &&
                !event.metaKey &&
                !event.altKey
              ) {
                // A typed character that is not a digit or a colon.
                event.preventDefault();
              }
            }}
            onBlur={() => setEditing(false)}
          />
          <div className="presets">
            {COUNTDOWN_PRESETS.map((seconds) => (
              <button
                key={seconds}
                type="button"
                className="preset"
                // Keep the focus in the input so it does not close first.
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => applyCountdown(seconds)}
              >
                {seconds / 60}
              </button>
            ))}
          </div>
        </>
      )}

      <div className="controls top">
        <button
          type="button"
          className="ctl"
          aria-label={countdown ? TEXT.toPomodoro : TEXT.toCountdown}
          title={countdown ? TEXT.toPomodoro : TEXT.toCountdown}
          onClick={() => engine.setMode(countdown ? "pomodoro" : "countdown")}
        >
          <SwapIcon />
        </button>
        <button type="button" className="ctl" aria-label={TEXT.stats} title={TEXT.stats} disabled>
          <ChartIcon />
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={TEXT.settings}
          title={TEXT.settings}
          onClick={() => settingsApi.open()}
        >
          <CogIcon />
        </button>
      </div>

      <div className="controls bottom">
        <button
          type="button"
          className="ctl primary"
          aria-label={playLabel(snapshot)}
          title={playLabel(snapshot)}
          onClick={() => engine.toggle()}
        >
          {running ? <PauseIcon /> : <PlayIcon />}
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={TEXT.skip}
          title={TEXT.skip}
          disabled={countdown}
          onClick={() => engine.skip()}
        >
          <SkipIcon />
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={TEXT.reset}
          title={TEXT.reset}
          onClick={() => engine.reset()}
        >
          <ResetIcon />
        </button>
      </div>
    </div>
  );
}
