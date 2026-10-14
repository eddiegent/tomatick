import { defineConfig } from "@playwright/test";

// Set PW_CHROMIUM_PATH to use a Chromium that is already installed.
const executablePath = process.env.PW_CHROMIUM_PATH;

export default defineConfig({
  testDir: "tests/ui",
  fullyParallel: true,
  reporter: process.env.CI ? [["list"], ["github"]] : "list",
  use: {
    baseURL: "http://localhost:1420",
    viewport: { width: 600, height: 900 },
    launchOptions: executablePath ? { executablePath } : {},
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
  },
});
