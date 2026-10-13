import { useEffect, useRef, useState, type CSSProperties } from "react";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import "./widget.css";
import { useAlertSounds } from "../alerts/useAlertSounds";
import { engine, formatClock, useSnapshot, type Snapshot } from "../engine/api";
import { formatDuration, parseDuration } from "../engine/duration";
import { settingsApi, useSettings } from "../settings/api";
import { statsApi } from "../stats/api";
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
import { progressOf, themeOf } from "./theme";

function playLabel(snapshot: Snapshot, t: TFunction): string {
  switch (snapshot.state) {
    case "running":
      return t("widget.pause");
    case "idle":
      return t("widget.start");
    default:
      return t("widget.resume");
  }
}

/** The always-on-top tomato. */
export function Widget() {
  const { t } = useTranslation();
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
    t(`phase.${theme}`),
    time,
    snapshot.label,
    paused ? (snapshot.state === "auto_paused" ? t("widget.autoPaused") : t("widget.paused")) : null,
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
          aria-label={t("widget.setCountdown")}
          title={t("widget.setCountdown")}
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
            aria-label={t("widget.countdownInput")}
            aria-invalid={invalid}
            title={t("widget.countdownHint")}
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
            {settings.settings.countdown_presets.map((seconds) => (
              <button
                key={seconds}
                type="button"
                className="preset"
                // Keep the focus in the input so it does not close first.
                onMouseDown={(event) => event.preventDefault()}
                onClick={() => applyCountdown(seconds)}
              >
                {formatDuration(seconds)}
              </button>
            ))}
          </div>
        </>
      )}

      <div className="controls top">
        <button
          type="button"
          className="ctl"
          aria-label={countdown ? t("widget.toPomodoro") : t("widget.toCountdown")}
          title={countdown ? t("widget.toPomodoro") : t("widget.toCountdown")}
          onClick={() => engine.setMode(countdown ? "pomodoro" : "countdown")}
        >
          <SwapIcon />
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={t("widget.stats")}
          title={t("widget.stats")}
          onClick={() => statsApi.open()}
        >
          <ChartIcon />
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={t("widget.settings")}
          title={t("widget.settings")}
          onClick={() => settingsApi.open()}
        >
          <CogIcon />
        </button>
      </div>

      <div className="controls bottom">
        <button
          type="button"
          className="ctl primary"
          aria-label={playLabel(snapshot, t)}
          title={playLabel(snapshot, t)}
          onClick={() => engine.toggle()}
        >
          {running ? <PauseIcon /> : <PlayIcon />}
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={t("widget.skip")}
          title={t("widget.skip")}
          disabled={countdown}
          onClick={() => engine.skip()}
        >
          <SkipIcon />
        </button>
        <button
          type="button"
          className="ctl"
          aria-label={t("widget.reset")}
          title={t("widget.reset")}
          onClick={() => engine.reset()}
        >
          <ResetIcon />
        </button>
      </div>
    </div>
  );
}
