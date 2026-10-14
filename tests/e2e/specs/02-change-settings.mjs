import { browser, expect } from "@wdio/globals";
import { call, currentSettings, invoke, widgetReady } from "../helpers.mjs";

/** A shortened cycle, so the history spec does not wait 25 minutes. */
export const CHANGED = {
  work_secs: 60,
  short_break_secs: 60,
  long_break_secs: 120,
  sessions_before_long: 2,
};

describe("changing the settings", () => {
  it("refuses a shortcut used twice", async () => {
    const { settings, exercises } = await currentSettings();
    settings.shortcuts.skip = settings.shortcuts.start_pause;
    const result = await invoke("settings_save", { settings, exercises });
    expect(result.err).toContain("more than one action");
  });

  it("refuses a language that has no file, and a daily goal out of range", async () => {
    const { settings, exercises } = await currentSettings();
    const language = await invoke("settings_save", { settings: { ...settings, language: "xx" }, exercises });
    expect(language.err).toContain("unknown language");
    const goal = await invoke("settings_save", { settings: { ...settings, daily_goal: 0 }, exercises });
    expect(goal.err).toContain("daily_goal");
  });

  it("saves new values, with the shortcuts in their canonical spelling", async () => {
    const { settings, exercises } = await currentSettings();
    settings.timer = { ...settings.timer, ...CHANGED };
    settings.daily_goal = 3;
    settings.language = "fr";
    settings.shortcuts = {
      start_pause: "ctrl+alt+j",
      skip: "CommandOrControl+Alt+K",
      toggle_widget: "CommandOrControl+Alt+L",
    };
    const saved = await call("settings_save", { settings, exercises });
    expect(saved.settings.shortcuts.start_pause).toBe("Control+Alt+J");
    expect(saved.settings.daily_goal).toBe(3);
  });

  it("registers the new shortcuts", async () => {
    const status = await call("desktop_status");
    expect(status.shortcuts[0].accelerator).toBe("Control+Alt+J");
  });

  it("switches the widget to French at once", async () => {
    await widgetReady();
    await browser.waitUntil(
      async () => ((await (await browser.$(".body")).getAttribute("title")) ?? "").startsWith("Travail"),
      { timeout: 10_000, timeoutMsg: "the widget did not switch to French" },
    );
    await expect(await browser.$("html")).toHaveAttribute("lang", "fr");
  });

  it("applies the new durations to the timer", async () => {
    await call("engine_reset");
    await browser.waitUntil(async () => (await call("engine_snapshot")).total_ms === 60_000, {
      timeout: 5000,
      timeoutMsg: "the timer kept the old duration",
    });
  });
});
