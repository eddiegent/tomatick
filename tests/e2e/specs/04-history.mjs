import { browser, expect } from "@wdio/globals";
import { call, widgetReady } from "../helpers.mjs";

const week = { period: "week", offset: 0, weekStart: 1 };

describe("the history", () => {
  it("records a pomodoro that runs to its end", async () => {
    await widgetReady();
    await call("engine_start");
    // The work phase lasts one minute (see 02-change-settings).
    await browser.waitUntil(async () => (await call("stats_get", week)).today.completed === 1, {
      timeout: 120_000,
      interval: 2000,
      timeoutMsg: "the pomodoro was not recorded",
    });
    const stats = await call("stats_get", week);
    expect(stats.totals.completed).toBe(1);
    expect(stats.totals.focus_secs).toBeGreaterThanOrEqual(59);
    expect(stats.streak.current).toBe(1);
    expect(stats.hours.reduce((a, b) => a + b, 0)).toBe(1);
    expect(stats.completion.completed).toBe(1);
  });

  it("leaves a phase running when the app is stopped", async () => {
    // Skip the break, start the next work phase, and end the session with
    // it running: the next launch must record it as interrupted.
    await call("engine_skip");
    const snapshot = await call("engine_snapshot");
    expect(snapshot.phase).toBe("work");
    await call("engine_start");
    await browser.waitUntil(async () => (await call("engine_snapshot")).state === "running", {
      timeout: 5000,
    });
    await expect(await browser.$(".widget")).toHaveAttribute("data-state", "running");
  });
});
