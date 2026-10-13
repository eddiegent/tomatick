import { useTranslation } from "react-i18next";
import { formatClock, type Snapshot } from "../engine/api";
import { themeOf } from "../widget/theme";

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
  const { t } = useTranslation();
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
      aria-label={kind === "countdown" ? t("overlay.countdownTitle") : t(`phase.${theme}`)}
    >
      <div className="overlay-card">
        <p className="overlay-eyebrow">
          {kind === "countdown" ? t("overlay.countdownEyebrow") : t("overlay.breakEyebrow")}
        </p>
        <h1 className="overlay-title">
          {kind === "countdown" ? t("overlay.countdownTitle") : t(`phase.${theme}`)}
        </h1>

        {kind === "break" && (
          <>
            <p className={time.length > 5 ? "overlay-clock long" : "overlay-clock"}>
              {time}
            </p>
            {snapshot.label && <p className="overlay-label">{snapshot.label}</p>}
            {snapshot.segment && (
              <p className="overlay-part">
                {t("overlay.part", { index: snapshot.segment.index + 1, count: snapshot.segment.count })}
              </p>
            )}
          </>
        )}

        <div className="overlay-actions">
          {kind === "countdown" ? (
            <button type="button" className="overlay-btn primary" onClick={actions.restart}>
              {t("overlay.restart")}
            </button>
          ) : idle ? (
            <button type="button" className="overlay-btn primary" onClick={actions.startBreak}>
              {t("overlay.startBreak")}
            </button>
          ) : (
            EXTEND_MINUTES.map((minutes) => (
              <button
                key={minutes}
                type="button"
                className="overlay-btn"
                onClick={() => actions.extend(minutes * 60)}
              >
                {t("overlay.extend", { minutes })}
              </button>
            ))
          )}
          {kind === "break" && (
            <button type="button" className="overlay-btn" onClick={actions.skipBreak}>
              {t("overlay.skipBreak")}
            </button>
          )}
          <button type="button" className="overlay-btn quiet" onClick={actions.dismiss}>
            {t("overlay.dismiss")}
          </button>
        </div>

        {preview && <p className="overlay-note">{t("overlay.preview")}</p>}
      </div>
    </div>
  );
}
