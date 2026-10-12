/** Longest countdown the engine accepts: 23:59:59. */
export const MAX_COUNTDOWN_SECS = 24 * 60 * 60 - 1;

/**
 * Parses what a person types as a duration and returns seconds, or `null`
 * when it is not a valid duration. Accepted forms:
 *
 * - `10` : minutes
 * - `1:30` : minutes and seconds
 * - `1:02:03` : hours, minutes and seconds
 * - `45s`, `5m`, `2h` : one unit
 */
export function parseDuration(text: string): number | null {
  const value = text.trim().toLowerCase();
  if (value === "") {
    return null;
  }

  const withUnit = /^(\d+)\s*([smh])$/.exec(value);
  if (withUnit) {
    const amount = Number(withUnit[1]);
    const factor = { s: 1, m: 60, h: 3600 }[withUnit[2] as "s" | "m" | "h"];
    return inRange(amount * factor);
  }

  if (!/^\d+(:\d{1,2}){0,2}$/.test(value)) {
    return null;
  }
  const parts = value.split(":").map(Number);
  if (parts.length === 1) {
    return inRange(parts[0] * 60);
  }
  if (parts.some((part, index) => index > 0 && part > 59)) {
    return null;
  }
  if (parts.length === 2) {
    return inRange(parts[0] * 60 + parts[1]);
  }
  return inRange(parts[0] * 3600 + parts[1] * 60 + parts[2]);
}

function inRange(seconds: number): number | null {
  return Number.isSafeInteger(seconds) && seconds >= 1 && seconds <= MAX_COUNTDOWN_SECS
    ? seconds
    : null;
}

/** Formats seconds the way `parseDuration` reads them back. */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");
  if (hours > 0) {
    return `${hours}:${pad(minutes)}:${pad(secs)}`;
  }
  return secs === 0 ? `${minutes}` : `${minutes}:${pad(secs)}`;
}
