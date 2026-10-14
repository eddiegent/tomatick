import { expect } from "@wdio/globals";
import { call, widgetReady } from "../helpers.mjs";

const week = { period: "week", offset: 0, weekStart: 1 };

describe("the history after a relaunch", () => {
  it("still has the finished pomodoro", async () => {
    await widgetReady();
    const stats = await call("stats_get", week);
    expect(stats.totals.completed).toBe(1);
    expect(stats.today.completed).toBe(1);
    expect(stats.streak.current).toBe(1);
  });

  it("recorded the phase that was running as interrupted", async () => {
    const stats = await call("stats_get", week);
    // One from the reset of a started phase in 01-first-run, one recovered
    // from the phase that was running when the app was stopped.
    expect(stats.completion.interrupted).toBe(2);
    // An interrupted phase is not a pomodoro.
    expect(stats.totals.completed).toBe(1);
  });

  it("does not restore the half-finished phase", async () => {
    const snapshot = await call("engine_snapshot");
    expect(snapshot.state).toBe("idle");
  });

  it("uses the daily goal that was saved", async () => {
    const stats = await call("stats_get", week);
    expect(stats.today.goal).toBe(3);
  });
});
