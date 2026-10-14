import { mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { test, type Page } from "@playwright/test";
import { defaultSettings, idleWork, openApp } from "../ui/mock";

/**
 * Renders the images of the documentation site (website/static/img/screenshots).
 *
 * They come from the interface with sample data, not from a real desktop, so
 * the window frame, the desktop behind the widget and the fonts of the system
 * are missing. Replace any of them with a real capture under the same name.
 */
const OUT = resolve(process.cwd(), "website/static/img/screenshots");
mkdirSync(OUT, { recursive: true });

const file = (name: string) => resolve(OUT, `${name}.png`);

/** The alert defaults of the app, which differ from the plain ones of the fake backend. */
const alert = (sound: string, overlay: boolean) => ({
  ...defaultSettings.alerts.work_end,
  sound: { kind: "builtin", value: sound },
  overlay,
});
const appSettings = {
  ...defaultSettings,
  alerts: {
    ...defaultSettings.alerts,
    work_end: alert("bell", true),
    short_break_end: alert("chime", false),
    long_break_end: alert("marimba", false),
    countdown_end: alert("beep", true),
  },
};

/** The widget in a given state, on a transparent background. */
async function widget(page: Page, name: string, snapshot: object, hover = false) {
  await openApp(page, { snapshot });
  const tomato = page.locator(".widget");
  if (hover) {
    await tomato.hover();
    await page.waitForTimeout(400);
  } else {
    await page.mouse.move(0, 0);
    await page.waitForTimeout(100);
  }
  await tomato.screenshot({ path: file(name), omitBackground: true });
}

test.describe("widget", () => {
  test.use({ viewport: { width: 200, height: 200 } });

  test("work", async ({ page }) => {
    await widget(page, "widget-work", {
      ...idleWork,
      state: "running",
      total_ms: 1_500_000,
      remaining_ms: 1_062_000,
    });
  });

  test("work with the controls", async ({ page }) => {
    await widget(
      page,
      "widget-controls",
      { ...idleWork, state: "running", total_ms: 1_500_000, remaining_ms: 1_062_000 },
      true,
    );
  });

  test("paused", async ({ page }) => {
    await widget(page, "widget-paused", {
      ...idleWork,
      state: "paused",
      total_ms: 1_500_000,
      remaining_ms: 640_000,
    });
  });

  test("short break", async ({ page }) => {
    await widget(page, "widget-short-break", {
      ...idleWork,
      state: "running",
      phase: "short_break",
      total_ms: 300_000,
      remaining_ms: 205_000,
      label: "Plank (30 s)",
    });
  });

  test("long break", async ({ page }) => {
    await widget(page, "widget-long-break", {
      ...idleWork,
      state: "running",
      phase: "long_break",
      total_ms: 900_000,
      remaining_ms: 640_000,
      segment: { index: 2, count: 3, total_ms: 300_000, remaining_ms: 130_000 },
      label: "Stretching (1 min)",
    });
  });

  test("countdown", async ({ page }) => {
    await widget(page, "widget-countdown", {
      mode: "countdown",
      state: "running",
      phase: null,
      total_ms: 600_000,
      remaining_ms: 372_000,
      segment: null,
      label: null,
      cycle: { position: 1, length: 4 },
    });
  });
});

test.describe("windows", () => {
  test("settings", async ({ page }) => {
    await page.setViewportSize({ width: 600, height: 900 });
    await openApp(page, { label: "settings", settings: appSettings });
    await page.getByRole("heading", { name: "Pomodoro cycle" }).waitFor();
    await page.screenshot({ path: file("settings") });
  });

  test("settings, alerts", async ({ page }) => {
    await page.setViewportSize({ width: 600, height: 900 });
    await openApp(page, { label: "settings", settings: appSettings });
    const heading = page.getByRole("heading", { name: "Alerts", exact: true });
    await heading.scrollIntoViewIfNeeded();
    await heading.evaluate((element) => element.scrollIntoView({ block: "start" }));
    await page.waitForTimeout(200);
    await page.screenshot({ path: file("settings-alerts") });
  });

  test("statistics", async ({ page }) => {
    await page.setViewportSize({ width: 600, height: 1000 });
    await openApp(page, { label: "stats" });
    await page.locator(".today .big").waitFor();
    await page.waitForTimeout(300);
    await page.screenshot({ path: file("statistics") });
  });

  test("break overlay", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await openApp(page, {
      label: "overlay",
      query: "?overlay=break",
      snapshot: {
        ...idleWork,
        state: "running",
        phase: "short_break",
        total_ms: 300_000,
        remaining_ms: 290_000,
        label: "Plank (30 s)",
      },
    });
    await page.getByRole("heading", { name: "Short break" }).waitFor();
    await page.waitForTimeout(300);
    await page.screenshot({ path: file("overlay-break") });
  });
});
