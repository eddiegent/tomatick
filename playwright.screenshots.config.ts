import { defineConfig } from "@playwright/test";

// Renders the images of the documentation site from the interface, with the
// same fake backend as the interface tests. Run it with `npm run docs:screenshots`.
// Set PW_CHROMIUM_PATH to use a Chromium that is already installed.
const executablePath = process.env.PW_CHROMIUM_PATH;

export default defineConfig({
  testDir: "tests/screenshots",
  workers: 1,
  reporter: "list",
  use: {
    baseURL: "http://localhost:1420",
    viewport: { width: 600, height: 900 },
    deviceScaleFactor: 2,
    launchOptions: executablePath ? { executablePath } : {},
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
  },
});
