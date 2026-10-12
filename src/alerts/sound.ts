import { invoke } from "@tauri-apps/api/core";
import type { SoundChoice } from "../settings/api";

const MIME_TYPES: Record<string, string> = {
  mp3: "audio/mpeg",
  ogg: "audio/ogg",
  wav: "audio/wav",
};

/** Object URLs of the custom sounds already read, by file name. */
const customUrls = new Map<string, string>();

/** The sound that is playing, so a new one can cut it off. */
let current: HTMLAudioElement | null = null;

async function urlFor(choice: SoundChoice): Promise<string> {
  if (choice.kind === "builtin") {
    return `/sounds/${choice.value}.wav`;
  }
  const cached = customUrls.get(choice.value);
  if (cached) {
    return cached;
  }
  const data = await invoke<ArrayBuffer>("sound_data", { file: choice.value });
  const extension = choice.value.split(".").pop()?.toLowerCase() ?? "";
  const url = URL.createObjectURL(
    new Blob([data], { type: MIME_TYPES[extension] ?? "audio/wav" }),
  );
  customUrls.set(choice.value, url);
  return url;
}

export function stopSound() {
  current?.pause();
  current = null;
}

/**
 * Plays an alert sound at the given volume (0 to 100) and resolves when it
 * has started. Rejects if the file is missing or cannot be decoded.
 */
export async function playSound(choice: SoundChoice, volume: number): Promise<void> {
  const url = await urlFor(choice);
  stopSound();
  const audio = new Audio(url);
  audio.volume = Math.min(1, Math.max(0, volume / 100));
  current = audio;
  audio.addEventListener("ended", () => {
    if (current === audio) {
      current = null;
    }
  });
  await audio.play();
}
