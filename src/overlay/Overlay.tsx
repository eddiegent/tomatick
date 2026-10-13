import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./overlay.css";
import { engine, useSnapshot, type Snapshot } from "../engine/api";
import { OverlayView, type OverlayActions, type OverlayKind } from "./OverlayView";

/** What the window was asked to show. */
interface Request {
  kind: OverlayKind;
  preview: boolean;
}

/** Sent by the Rust side when the overlay is already open. */
const OPEN_EVENT = "overlay:open";

function parseKind(value: string | null): OverlayKind {
  return value === "countdown" ? "countdown" : "break";
}

function initialRequest(): Request {
  const query = new URLSearchParams(window.location.search);
  return {
    kind: parseKind(query.get("overlay")),
    preview: query.get("preview") === "1",
  };
}

/** Sample content for the preview, so it never reads or changes the timer. */
function sampleSnapshot(kind: OverlayKind): Snapshot {
  const base = {
    state: "running" as const,
    segment: null,
    cycle: { position: 1, length: 4 },
  };
  if (kind === "countdown") {
    return {
      ...base,
      mode: "countdown",
      state: "idle",
      phase: null,
      total_ms: 10 * 60_000,
      remaining_ms: 10 * 60_000,
      label: null,
    };
  }
  return {
    ...base,
    mode: "pomodoro",
    phase: "short_break",
    total_ms: 5 * 60_000,
    remaining_ms: 5 * 60_000,
    label: "Plank (30 s)",
  };
}

/** Whether what the overlay is about no longer holds. */
function isStale(kind: OverlayKind, snapshot: Snapshot): boolean {
  if (kind === "countdown") {
    // The countdown was started again, or the user went back to Pomodoro.
    return snapshot.mode !== "countdown" || snapshot.state !== "idle";
  }
  // The break is over or was skipped: a work phase is next, or the mode changed.
  return (
    snapshot.mode !== "pomodoro" ||
    snapshot.phase === null ||
    snapshot.phase === "work"
  );
}

function closeWindow() {
  try {
    void getCurrentWindow().close();
  } catch {
    window.close();
  }
}

/** The full-screen overlay window. */
export function Overlay() {
  const [request, setRequest] = useState<Request>(initialRequest);
  const live = useSnapshot();
  const root = useRef<HTMLDivElement>(null);

  // Another alert while the overlay is open changes what it shows.
  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    listen<Request>(OPEN_EVENT, (event) => setRequest(event.payload)).then((fn) => {
      if (active) {
        unlisten = fn;
      } else {
        fn();
      }
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  const { kind, preview } = request;
  const snapshot = useMemo(
    () => (preview ? sampleSnapshot(kind) : live),
    [preview, kind, live],
  );

  const stale = !preview && snapshot !== null && isStale(kind, snapshot);
  useEffect(() => {
    if (stale) {
      closeWindow();
    }
  }, [stale]);

  // Escape dismisses. The focus goes to the layer, not to a button, so a stray
  // Enter or Space does not skip the break.
  useEffect(() => {
    root.current?.focus();
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        closeWindow();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const run = useCallback(
    (command: () => Promise<void>, closeAfter: boolean) => {
      if (preview) {
        closeWindow();
        return;
      }
      command()
        .catch(() => undefined)
        .finally(() => {
          if (closeAfter) {
            closeWindow();
          }
        });
    },
    [preview],
  );

  const actions: OverlayActions = useMemo(
    () => ({
      startBreak: () => run(engine.start, false),
      skipBreak: () => run(engine.skipAndStart, true),
      extend: (seconds) => run(() => engine.extend(seconds), false),
      restart: () => run(engine.start, true),
      dismiss: closeWindow,
    }),
    [run],
  );

  if (!snapshot) {
    return null;
  }

  return (
    <div className="overlay-root" ref={root} tabIndex={-1}>
      <OverlayView kind={kind} snapshot={snapshot} preview={preview} actions={actions} />
    </div>
  );
}
