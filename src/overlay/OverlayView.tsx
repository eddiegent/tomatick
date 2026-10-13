import { formatClock, type Snapshot } from "../engine/api";
import { themeOf, type Theme } from "../widget/theme";

// English only for now. The text moves to the translation files in
// milestone 10.
const TEXT = {
  phase: {
    work: "Work",
    short_break: "Short break",
    long_break: "Long break",
    countdown: "Countdown",
  } satisfies Record<Theme, string>,
  breakEyebrow: "Time for a break",
  countdownTitle: "Time's up",
  countdownEyebrow: "Countdown finished",
  part: (index: number, count: number) => `Part ${index + 1} of ${count}`,
  startBreak: "Start break",
  skipBreak: "Skip break",
  extend: (minutes: number) => `Extend +${minutes} min`,
  restart: "Restart",
  dismiss: "Dismiss",
  preview: "Preview. The buttons only close this window.",
};

/** Extra time the Extend buttons add, in minutes. */
export const EXTEND_MINUTES = [1, 5];

export type OverlayKind = "break" | "countdown";

/** What the buttons do. The preview points all of them at `dismiss`. */
export interface OverlayActions {
  startBreak: () => void;
  skipBreak: () => void;
  extend: (seconds: number) => void;
  restart: () => void;
  dismiss: () => void;
}

export interface OverlayViewProps {
  kind: OverlayKind;
  snapshot: Snapshot;
  preview: boolean;
  actions: OverlayActions;
}

/**
 * The dimmed full-screen layer. It only draws what it is given, so it can be
 * rendered with any snapshot.
 */
export function OverlayView({ kind, snapshot, preview, actions }: OverlayViewProps) {
  const theme = kind === "countdown" ? "countdown" : themeOf(snapshot);
  const idle = snapshot.state === "idle";
  const time = formatClock(snapshot.remaining_ms);

  return (
    <div
      className="overlay"
      data-theme={theme}
      data-kind={kind}
      role="dialog"
      aria-modal="true"
      aria-label={kind === "countdown" ? TEXT.countdownTitle : TEXT.phase[theme]}
    >
      <div className="overlay-card">
        <p className="overlay-eyebrow">
          {kind === "countdown" ? TEXT.countdownEyebrow : TEXT.breakEyebrow}
        </p>
        <h1 className="overlay-title">
          {kind === "countdown" ? TEXT.countdownTitle : TEXT.phase[theme]}
        </h1>

        {kind === "break" && (
          <>
            <p className={time.length > 5 ? "overlay-clock long" : "overlay-clock"}>
              {time}
            </p>
            {snapshot.label && <p className="overlay-label">{snapshot.label}</p>}
            {snapshot.segment && (
              <p className="overlay-part">
                {TEXT.part(snapshot.segment.index, snapshot.segment.count)}
              </p>
            )}
          </>
        )}

        <div className="overlay-actions">
          {kind === "countdown" ? (
            <button type="button" className="overlay-btn primary" onClick={actions.restart}>
              {TEXT.restart}
            </button>
          ) : idle ? (
            <button type="button" className="overlay-btn primary" onClick={actions.startBreak}>
              {TEXT.startBreak}
            </button>
          ) : (
            EXTEND_MINUTES.map((minutes) => (
              <button
                key={minutes}
                type="button"
                className="overlay-btn"
                onClick={() => actions.extend(minutes * 60)}
              >
                {TEXT.extend(minutes)}
              </button>
            ))
          )}
          {kind === "break" && (
            <button type="button" className="overlay-btn" onClick={actions.skipBreak}>
              {TEXT.skipBreak}
            </button>
          )}
          <button type="button" className="overlay-btn quiet" onClick={actions.dismiss}>
            {TEXT.dismiss}
          </button>
        </div>

        {preview && <p className="overlay-note">{TEXT.preview}</p>}
      </div>
    </div>
  );
}
