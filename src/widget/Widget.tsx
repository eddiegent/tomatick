import { useEffect, type CSSProperties } from "react";
import "./widget.css";
import { engine, formatClock, useSnapshot, type Snapshot } from "../engine/api";
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
import { WIDGET_PIXELS, useWidgetSize, type WidgetSize } from "./size";
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
  settings: "Settings (coming soon)",
  paused: "Paused",
  autoPaused: "Paused by the system",
};

const SIZE_KEYS: Record<string, WidgetSize> = {
  "1": "small",
  "2": "medium",
  "3": "large",
};

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
  const [size, setSize] = useWidgetSize();

  // Temporary shortcut to try the three sizes: press 1, 2 or 3. It goes away
  // when the settings dialog lands (milestone 4).
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.ctrlKey || event.metaKey || event.altKey) {
        return;
      }
      const next = SIZE_KEYS[event.key];
      if (next) {
        setSize(next);
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setSize]);

  // The browser context menu does not belong on a desktop widget.
  useEffect(() => {
    const block = (event: Event) => event.preventDefault();
    window.addEventListener("contextmenu", block);
    return () => window.removeEventListener("contextmenu", block);
  }, []);

  if (!snapshot) {
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
      className="widget"
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
        <span className={time.length > 5 ? "time long" : "time"}>{time}</span>
        {snapshot.label && <span className="label">{snapshot.label}</span>}
        {paused && (
          <span className="paused-mark">
            {snapshot.state === "auto_paused" ? <MoonIcon /> : <PauseIcon />}
          </span>
        )}
      </div>

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
          disabled
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
