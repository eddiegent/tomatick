import { useEffect, useRef } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { onEngineEvent } from "../engine/api";
import type { AlertSettings } from "../settings/api";
import { playSound } from "./sound";

/**
 * Plays the sound of each alert the engine raises, and a light cue when a
 * long break moves to its next segment. Only the widget window
 * uses this, so a sound is played once. It always reads the latest settings
 * without listening again.
 */
export function useAlertSounds(alerts: AlertSettings | undefined) {
  const latest = useRef(alerts);
  useEffect(() => {
    latest.current = alerts;
  }, [alerts]);

  useEffect(() => {
    let active = true;
    let unlisten: UnlistenFn | undefined;

    onEngineEvent((event) => {
      if (event.type !== "alert" && event.type !== "segment_changed") {
        return;
      }
      const settings = latest.current;
      if (!settings || settings.muted || settings.volume === 0) {
        return;
      }
      // Moving to the next segment of a long break is only a light cue: the
      // sound of the end of a short break, with no notification or overlay.
      const config = settings[event.type === "alert" ? event.event : "short_break_end"];
      if (!config.sound_enabled) {
        return;
      }
      playSound(config.sound, settings.volume).catch((error) =>
        console.warn("Tomatick: cannot play the alert sound", error),
      );
    }).then((fn) => {
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
}
