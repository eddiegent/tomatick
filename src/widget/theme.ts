import type { Snapshot } from "../engine/api";

export type Theme = "work" | "short_break" | "long_break" | "countdown";

/** Which color scheme the tomato wears. */
export function themeOf(snapshot: Snapshot): Theme {
  if (snapshot.mode === "countdown") {
    return "countdown";
  }
  return snapshot.phase ?? "work";
}

/**
 * Progress through the current phase, from 0 to 1. During a long break the
 * ring follows the current segment, not the whole break.
 */
export function progressOf(snapshot: Snapshot): number {
  const total = snapshot.segment ? snapshot.segment.total_ms : snapshot.total_ms;
  const remaining = snapshot.segment
    ? snapshot.segment.remaining_ms
    : snapshot.remaining_ms;
  if (total <= 0) {
    return 0;
  }
  return Math.min(1, Math.max(0, 1 - remaining / total));
}
