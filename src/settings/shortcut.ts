/** Helpers for the global shortcut fields of the settings dialog. */

/** Mirrors `ShortcutSettings` in src-tauri/src/shortcuts.rs. */
export interface Shortcuts {
  start_pause: string;
  skip: string;
  toggle_widget: string;
}

export type ShortcutAction = keyof Shortcuts;

/** The defaults, same as the Rust side. */
export const DEFAULT_SHORTCUTS: Shortcuts = {
  start_pause: "CommandOrControl+Alt+P",
  skip: "CommandOrControl+Alt+N",
  toggle_widget: "CommandOrControl+Alt+T",
};

export const isMac = (): boolean => /mac/i.test(navigator.platform || navigator.userAgent);

const NAMED_KEYS: Record<string, string> = {
  Space: "Space",
  Enter: "Enter",
  Tab: "Tab",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
  Insert: "Insert",
  Delete: "Delete",
};

/** The key part of a shortcut for a `KeyboardEvent.code`, or `null`. */
export function keyName(code: string): string | null {
  const letter = /^Key([A-Z])$/.exec(code);
  if (letter) {
    return letter[1];
  }
  const digit = /^Digit(\d)$/.exec(code);
  if (digit) {
    return digit[1];
  }
  const fn = /^F(\d{1,2})$/.exec(code);
  if (fn && Number(fn[1]) >= 1 && Number(fn[1]) <= 24) {
    return `F${Number(fn[1])}`;
  }
  return NAMED_KEYS[code] ?? null;
}

export interface KeyPress {
  code: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
}

/**
 * The shortcut for a key press: the text, `"needs-modifier"` when only a
 * plain key was pressed, or `null` while only modifiers are held down.
 */
export function shortcutFromKeys(press: KeyPress, mac: boolean): string | "needs-modifier" | null {
  const key = keyName(press.code);
  if (key === null) {
    return null;
  }
  const modifiers: string[] = [];
  if (mac ? press.metaKey : press.ctrlKey) {
    modifiers.push("CommandOrControl");
  }
  if (mac ? press.ctrlKey : false) {
    modifiers.push("Control");
  }
  if (!mac && press.metaKey) {
    modifiers.push("Super");
  }
  if (press.altKey) {
    modifiers.push("Alt");
  }
  if (press.shiftKey) {
    modifiers.push("Shift");
  }
  return modifiers.length === 0 ? "needs-modifier" : [...modifiers, key].join("+");
}

/** A shortcut as the user's keyboard names it. */
export function displayShortcut(text: string, mac: boolean): string {
  return text
    .split("+")
    .map((part) => {
      switch (part) {
        case "CommandOrControl":
          return mac ? "Cmd" : "Ctrl";
        case "Control":
          return "Ctrl";
        case "Super":
          return mac ? "Cmd" : "Win";
        case "Alt":
          return mac ? "Option" : "Alt";
        default:
          return part;
      }
    })
    .join("+");
}
