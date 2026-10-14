import { browser, expect } from "@wdio/globals";
import { call, currentSettings, widgetReady } from "../helpers.mjs";

describe("first launch", () => {
  it("shows the widget with a full work phase", async () => {
    const widget = await widgetReady();
    await expect(widget).toHaveAttribute("data-state", "idle");
    await expect(widget).toHaveAttribute("data-theme", "work");
    await expect(await browser.$(".time")).toHaveText("25:00");
    await expect(await browser.$(".cycle")).toHaveText("1/4");
  });

  it("starts with the default settings", async () => {
    const { settings } = await currentSettings();
    expect(settings.daily_goal).toBe(8);
    expect(settings.language).toBe("system");
    expect(settings.launch_at_login).toBe(false);
    expect(settings.timer.work_secs).toBe(1500);
    expect(settings.shortcuts.start_pause).toBe("CommandOrControl+Alt+P");
  });

  it("reports the state of the three global shortcuts", async () => {
    const status = await call("desktop_status");
    expect(status.shortcuts.map((s) => s.action)).toEqual(["start_pause", "skip", "toggle_widget"]);
  });

  it("runs and pauses the timer, and the widget follows", async () => {
    const widget = await widgetReady();
    await call("engine_toggle");
    await browser.waitUntil(async () => (await widget.getAttribute("data-state")) === "running", {
      timeout: 5000,
    });
    // The clock moves while it runs.
    await browser.waitUntil(async () => (await (await browser.$(".time")).getText()) !== "25:00", {
      timeout: 10_000,
      timeoutMsg: "the clock did not move",
    });
    await call("engine_toggle");
    await browser.waitUntil(async () => (await widget.getAttribute("data-state")) === "paused", {
      timeout: 5000,
    });
    await call("engine_reset");
    await browser.waitUntil(async () => (await widget.getAttribute("data-state")) === "idle", {
      timeout: 5000,
    });
  });

  it("has an empty history", async () => {
    const stats = await call("stats_get", { period: "week", offset: 0, weekStart: 1 });
    expect(stats.totals.completed).toBe(0);
    expect(stats.today.goal).toBe(8);
    expect(stats.buckets).toHaveLength(7);
  });
});
