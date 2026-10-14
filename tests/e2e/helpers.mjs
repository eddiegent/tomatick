import { browser } from "@wdio/globals";

/**
 * Calls a Tauri command from the page, as the interface does. Resolves to
 * `{ ok: value }`, or `{ err: message }` when the command fails.
 */
export async function invoke(command, args = {}) {
  return browser.executeAsync(
    (name, payload, done) => {
      window.__TAURI_INTERNALS__
        .invoke(name, payload)
        .then((value) => done({ ok: value }))
        .catch((error) => done({ err: String(error) }));
    },
    command,
    args,
  );
}

/** Like `invoke`, but throws when the command fails. */
export async function call(command, args = {}) {
  const result = await invoke(command, args);
  if ("err" in result) {
    throw new Error(`${command} failed: ${result.err}`);
  }
  return result.ok;
}

/** Waits until the widget is on screen. */
export async function widgetReady() {
  const widget = await browser.$(".widget");
  await widget.waitForExist({ timeout: 30_000 });
  return widget;
}

/** The settings of the app, as saved, ready to change. */
export async function currentSettings() {
  const { settings, exercises } = await call("settings_get");
  return { settings, exercises };
}
