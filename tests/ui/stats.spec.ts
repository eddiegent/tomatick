import { expect, test } from "@playwright/test";
import { calls, defaultSettings, emit, openApp, weekStats } from "./mock";

const stats = { label: "stats" };

test("the statistics window shows the week", async ({ page }) => {
  await openApp(page, stats);
  await expect(page.locator(".today .big")).toHaveText("3 of 8 pomodoros");
  await expect(page.locator(".range")).toContainText("Oct 12");
  await expect(page.locator(".range")).toContainText("Oct 18, 2026");
  await expect(page.getByRole("heading", { name: "Streak" })).toBeVisible();
  await expect(page.getByText("3 days")).toBeVisible();
  await expect(page.getByText("5 days")).toBeVisible();
  // 9 of 12 work phases completed.
  await expect(page.getByText("75%")).toBeVisible();
  await expect(page.getByText("9 completed, 2 skipped, 1 interrupted")).toBeVisible();
  await expect(page.locator("svg.chart").first().locator("rect.bar")).toHaveCount(7);
  await expect(page.locator("svg.chart").nth(1).locator("rect.bar")).toHaveCount(24);
});

test("the period tabs and the arrows query the matching period", async ({ page }) => {
  await openApp(page, stats);
  const next = page.getByRole("button", { name: "Next" });
  await expect(next).toBeDisabled();

  await page.getByRole("button", { name: "Previous" }).click();
  await expect(next).toBeEnabled();
  await page.getByRole("radio", { name: "Month" }).click();
  await expect(next).toBeDisabled();

  const gets = (await calls(page)).filter((c) => c.cmd === "stats_get").map((c) => c.args);
  expect(gets[0]).toMatchObject({ period: "week", offset: 0 });
  expect(gets).toContainEqual(expect.objectContaining({ period: "week", offset: -1 }));
  expect(gets.at(-1)).toMatchObject({ period: "month", offset: 0 });
});

test("the chart can show the focus time", async ({ page }) => {
  await openApp(page, stats);
  await page.getByRole("radio", { name: "Focus time" }).click();
  await expect(page.getByRole("heading", { name: "Focus time" }).first()).toBeVisible();
  // Tooltip of the busiest day: 6000 s is 1 h 40 min.
  await expect(page.locator("svg.chart").first().locator("title").nth(1)).toContainText("1 h 40 min");
});

test("an empty period says so", async ({ page }) => {
  await openApp(page, {
    ...stats,
    stats: {
      ...weekStats,
      today: { completed: 0, focus_secs: 0, goal: 8 },
      buckets: weekStats.buckets.map((b) => ({ ...b, completed: 0, focus_secs: 0 })),
      streak: { current: 0, longest: 0 },
      completion: { completed: 0, skipped: 0, interrupted: 0 },
      hours: Array(24).fill(0),
      totals: { completed: 0, focus_secs: 0, active_days: 0, average_focus_secs: 0 },
    },
  });
  await expect(page.getByText("No pomodoros in this period yet.")).toBeVisible();
  await expect(page.getByText("No work phases yet")).toBeVisible();
});

test("a finished phase refreshes the window", async ({ page }) => {
  await openApp(page, stats);
  const before = (await calls(page)).filter((c) => c.cmd === "stats_get").length;
  await emit(page, "history:changed", null);
  await expect
    .poll(async () => (await calls(page)).filter((c) => c.cmd === "stats_get").length)
    .toBeGreaterThan(before);
});

test.describe("regional formats", () => {
  test.use({ locale: "en-GB" });

  test("English on a British system writes the day first", async ({ page }) => {
    await openApp(page, stats);
    await expect(page.locator(".range")).toContainText("12 Oct");
    await expect(page.locator(".range")).toContainText("18 Oct 2026");
  });

  test("French on a British system uses French formats", async ({ page }) => {
    await openApp(page, { ...stats, settings: { ...defaultSettings, language: "fr" } });
    await expect(page.locator(".range")).toContainText("12 oct.");
  });
});

test("the statistics follow the language", async ({ page }) => {
  await openApp(page, { ...stats, settings: { ...defaultSettings, language: "fr" } });
  await expect(page.locator(".today .big")).toHaveText("3 sur 8 pomodoros");
  await expect(page.getByRole("heading", { name: "Série" })).toBeVisible();
  await expect(page.getByText("3 jours")).toBeVisible();
  await expect(page.getByRole("radio", { name: "Semaine" })).toBeVisible();
  await expect(page).toHaveTitle("Statistiques");
  // French writes a space before the percent sign.
  await expect(page.getByText(/^75\s%$/)).toBeVisible();
});
