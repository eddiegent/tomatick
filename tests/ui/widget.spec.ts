import { expect, test } from "@playwright/test";
import { calls, defaultSettings, emit, idleWork, openApp } from "./mock";

test("the widget shows the phase and starts and pauses the timer", async ({ page }) => {
  await openApp(page);
  const widget = page.locator(".widget");
  await expect(widget).toHaveAttribute("data-state", "idle");
  await expect(widget).toHaveAttribute("data-theme", "work");
  await expect(page.locator(".time")).toHaveText("25:00");
  await expect(page.locator(".cycle")).toHaveText("1/4");
  await expect(page.locator(".body")).toHaveAttribute("title", /Work \| 25:00/);

  await widget.hover();
  await page.getByRole("button", { name: "Start" }).click();
  await expect(widget).toHaveAttribute("data-state", "running");
  await page.getByRole("button", { name: "Pause", exact: true }).click();
  await expect(widget).toHaveAttribute("data-state", "paused");

  const sent = (await calls(page)).filter((c) => c.cmd === "engine_toggle");
  expect(sent).toHaveLength(2);
});

test("the controls send the matching commands", async ({ page }) => {
  await openApp(page);
  // The controls only appear while the pointer is over the widget.
  await page.locator(".widget").hover();
  await page.getByRole("button", { name: "Skip to the next phase" }).click();
  await page.getByRole("button", { name: "Reset" }).click();
  await page.getByRole("button", { name: "Switch to countdown" }).click();
  await page.getByRole("button", { name: "Statistics" }).click();
  await page.getByRole("button", { name: "Settings" }).click();

  const names = (await calls(page)).map((c) => c.cmd);
  expect(names).toEqual(
    expect.arrayContaining([
      "engine_skip",
      "engine_reset",
      "engine_set_mode",
      "stats_open",
      "settings_open",
    ]),
  );
  const mode = (await calls(page)).find((c) => c.cmd === "engine_set_mode");
  expect(mode?.args).toEqual({ mode: "countdown" });
});

test("a pause by the system is shown as such", async ({ page }) => {
  await openApp(page, { snapshot: { ...idleWork, state: "auto_paused" } });
  await expect(page.locator(".body")).toHaveAttribute("title", /Paused by the system/);
  await expect(page.locator(".paused-mark")).toBeVisible();
});

test("the countdown can be edited with a preset", async ({ page }) => {
  await openApp(page, {
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
  await expect(page.locator(".widget")).toHaveAttribute("data-theme", "countdown");
  await page.locator(".widget").hover();
  await page.getByRole("button", { name: "Click to set the countdown" }).click();
  await page.getByRole("button", { name: "15", exact: true }).click();
  const sent = (await calls(page)).find((c) => c.cmd === "engine_set_countdown");
  expect(sent?.args).toEqual({ seconds: 900 });
});

test("the widget follows the language setting", async ({ page }) => {
  await openApp(page);
  await expect(page.getByRole("button", { name: "Start", includeHidden: true })).toHaveCount(1);
  await emit(page, "settings:changed", {
    settings: { ...defaultSettings, language: "fr" },
    exercises: [],
  });
  await expect(page.getByRole("button", { name: "Démarrer", includeHidden: true })).toHaveCount(1);
  await expect(page.locator(".body")).toHaveAttribute("title", /Travail/);
  await expect(page.locator("html")).toHaveAttribute("lang", "fr");
});
