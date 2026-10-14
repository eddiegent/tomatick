import { expect, test } from "@playwright/test";
import { calls, defaultSettings, okStatus, openApp } from "./mock";

const settings = { label: "settings" };

async function lastSave(page: import("@playwright/test").Page) {
  const saves = (await calls(page)).filter((c) => c.cmd === "settings_save");
  return saves.at(-1)?.args as { settings: typeof defaultSettings } | undefined;
}

test("the form shows the stored settings", async ({ page }) => {
  await openApp(page, settings);
  await expect(page.getByLabel("Work (minutes)")).toHaveValue("25");
  await expect(page.getByLabel("Daily goal (pomodoros)")).toHaveValue("8");
  await expect(page.getByLabel("Language")).toHaveValue("en");
  await expect(page.getByRole("button", { name: /Start or pause: Ctrl\+Alt\+P|Start or pause: Cmd\+Alt\+P/ })).toBeVisible();
});

test("a value out of range is refused before saving", async ({ page }) => {
  await openApp(page, settings);
  await page.getByLabel("Work (minutes)").fill("0");
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText("Work must be a whole number from 1 to 180 minutes.")).toBeVisible();
  expect(await lastSave(page)).toBeUndefined();
});

test("saving sends every field", async ({ page }) => {
  await openApp(page, settings);
  await page.getByLabel("Work (minutes)").fill("30");
  await page.getByLabel("Daily goal (pomodoros)").fill("5");
  await page.getByLabel("Launch Tomatick when I log in").check();
  await page.getByRole("button", { name: "Save" }).click();

  await expect.poll(() => lastSave(page)).toBeDefined();
  const saved = (await lastSave(page))!.settings;
  expect(saved.timer.work_secs).toBe(1800);
  expect(saved.daily_goal).toBe(5);
  expect(saved.launch_at_login).toBe(true);
  expect(saved.shortcuts).toEqual(defaultSettings.shortcuts);
  // On success the window closes itself.
  await expect
    .poll(async () => (await calls(page)).some((c) => c.cmd === "plugin:window|close"))
    .toBe(true);
});

test("a shortcut is rebound by pressing the new keys", async ({ page }) => {
  await openApp(page, settings);
  const field = page.locator(".shortcut-row").first().locator("button.shortcut");
  await field.click();
  await expect(field).toHaveText("Press the keys");

  // A plain key is not enough.
  await page.keyboard.press("p");
  await expect(page.getByText("Hold Ctrl, Alt, Shift or Cmd together with the key.")).toBeVisible();

  await page.keyboard.press("Control+Shift+K");
  await expect(field).toHaveText(/Ctrl\+Shift\+K/);
  await page.getByRole("button", { name: "Save" }).click();
  await expect.poll(() => lastSave(page)).toBeDefined();
  expect((await lastSave(page))!.settings.shortcuts.start_pause).toBe("CommandOrControl+Shift+K");
});

test("escape while recording cancels the recording, not the window", async ({ page }) => {
  await openApp(page, settings);
  const field = page.locator(".shortcut-row").first().locator("button.shortcut");
  await field.click();
  await page.keyboard.press("Escape");
  await expect(field).not.toHaveText("Press the keys");
  expect((await calls(page)).some((c) => c.cmd === "plugin:window|close")).toBe(false);
});

test("a shortcut can be cleared or reset to its default", async ({ page }) => {
  await openApp(page, settings);
  const row = page.locator(".shortcut-row").nth(1);
  await row.getByRole("button", { name: "Clear" }).click();
  await expect(row.locator("button.shortcut")).toHaveText("Not set");
  await row.getByRole("button", { name: "Default" }).click();
  await expect(row.locator("button.shortcut")).toHaveText(/\+Alt\+N$/);
});

test("a shortcut the system refused is reported and the window stays open", async ({ page }) => {
  await openApp(page, {
    ...settings,
    statusAfterSave: {
      ...okStatus,
      shortcuts: [
        okStatus.shortcuts[0],
        { action: "skip", accelerator: "CommandOrControl+Alt+N", error: "already registered" },
        okStatus.shortcuts[2],
      ],
    },
  });
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText("Could not register this shortcut: already registered")).toBeVisible();
  await expect(page.getByText(/Saved, but some desktop settings did not apply/)).toBeVisible();
  expect((await calls(page)).some((c) => c.cmd === "plugin:window|close")).toBe(false);
});

test("an error from the backend is shown and the window stays open", async ({ page }) => {
  await openApp(page, { ...settings, saveError: "shortcut is used for more than one action" });
  await page.getByRole("button", { name: "Save" }).click();
  await expect(page.getByText("shortcut is used for more than one action")).toBeVisible();
  await expect(page.getByRole("button", { name: "Save" })).toBeEnabled();
});

test("choosing French switches the interface once saved", async ({ page }) => {
  await openApp(page, settings);
  await page.getByLabel("Language").selectOption("fr");
  await page.getByRole("button", { name: "Save" }).click();
  await expect.poll(() => lastSave(page)).toBeDefined();
  expect((await lastSave(page))!.settings.language).toBe("fr");
  await expect(page).toHaveTitle("Réglages");
  await expect(page.getByRole("button", { name: "Annuler" })).toBeVisible();
});

test("the languages come from the locale files", async ({ page }) => {
  await openApp(page, settings);
  const options = await page.getByLabel("Language").locator("option").allTextContents();
  expect(options).toEqual(["Same as the system", "English", "Français"]);
});
