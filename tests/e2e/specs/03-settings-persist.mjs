import { browser, expect } from "@wdio/globals";
import { call, currentSettings, widgetReady } from "../helpers.mjs";

// A new launch of the app: everything saved by the previous spec must be back.
describe("after a relaunch", () => {
  it("has the saved settings", async () => {
    await widgetReady();
    const { settings } = await currentSettings();
    expect(settings.daily_goal).toBe(3);
    expect(settings.language).toBe("fr");
    expect(settings.timer.work_secs).toBe(60);
    expect(settings.timer.sessions_before_long).toBe(2);
    expect(settings.shortcuts.start_pause).toBe("Control+Alt+J");
  });

  it("starts in French", async () => {
    await browser.waitUntil(
      async () => ((await (await browser.$(".body")).getAttribute("title")) ?? "").startsWith("Travail"),
      { timeout: 10_000, timeoutMsg: "the widget did not start in French" },
    );
  });

  it("registers the saved shortcuts again", async () => {
    const status = await call("desktop_status");
    expect(status.shortcuts[0].accelerator).toBe("Control+Alt+J");
  });

  it("uses the saved durations", async () => {
    expect((await call("engine_snapshot")).total_ms).toBe(60_000);
  });
});
