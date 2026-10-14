import { expect, test } from "@playwright/test";
import { calls, idleWork, openApp } from "./mock";

const shortBreak = {
  ...idleWork,
  state: "running",
  phase: "short_break",
  total_ms: 300_000,
  remaining_ms: 290_000,
  label: "Plank (30 s)",
};

test("the break overlay shows the exercise and extends the break", async ({ page }) => {
  await openApp(page, { label: "overlay", snapshot: shortBreak, query: "?overlay=break" });
  await expect(page.getByRole("heading", { name: "Short break" })).toBeVisible();
  await expect(page.getByText("Time for a break")).toBeVisible();
  await expect(page.getByText("Plank (30 s)")).toBeVisible();
  await page.getByRole("button", { name: "Extend +5 min" }).click();
  const sent = (await calls(page)).find((c) => c.cmd === "engine_extend");
  expect(sent?.args).toEqual({ seconds: 300 });
});

test("the countdown overlay offers a restart", async ({ page }) => {
  await openApp(page, {
    label: "overlay",
    query: "?overlay=countdown",
    snapshot: {
      mode: "countdown",
      state: "idle",
      phase: null,
      total_ms: 600_000,
      remaining_ms: 600_000,
      segment: null,
      label: null,
      cycle: { position: 1, length: 4 },
    },
  });
  await expect(page.getByRole("heading", { name: "Time's up" })).toBeVisible();
  await page.getByRole("button", { name: "Restart" }).click();
  expect((await calls(page)).some((c) => c.cmd === "engine_start")).toBe(true);
});

test("the preview says that its buttons do nothing", async ({ page }) => {
  await openApp(page, { label: "overlay", query: "?overlay=break&preview=1" });
  await expect(page.getByText("Preview. The buttons only close this window.")).toBeVisible();
  await expect(page.getByText("Plank (30 s)")).toBeVisible();
});
