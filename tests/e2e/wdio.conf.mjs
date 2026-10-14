import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const exe = process.platform === "win32" ? ".exe" : "";

/** The app under test. Build it first: `npm run tauri build -- --debug --no-bundle`. */
const application =
  process.env.TOMATICK_BIN ??
  path.resolve(here, "../../src-tauri/target/debug", `tomatick${exe}`);

// The app keeps its database in the user data folder. Point every folder it
// could use at a scratch one, so the tests never touch the real data. The
// same path is used by every spec, so a spec sees what the one before it
// saved: that is how persistence across a relaunch is tested.
const dataDir = path.join(os.tmpdir(), "tomatick-e2e-data");
Object.assign(process.env, {
  XDG_DATA_HOME: path.join(dataDir, "data"),
  XDG_CONFIG_HOME: path.join(dataDir, "config"),
  XDG_CACHE_HOME: path.join(dataDir, "cache"),
  APPDATA: path.join(dataDir, "appdata"),
  LOCALAPPDATA: path.join(dataDir, "localappdata"),
});

let tauriDriver;
let stopping = false;

function stopDriver() {
  stopping = true;
  tauriDriver?.kill();
}

export const config = {
  host: "127.0.0.1",
  port: 4444,
  // Alphabetical order matters: each spec is one launch of the app.
  specs: ["./specs/*.mjs"],
  maxInstances: 1,
  capabilities: [
    {
      maxInstances: 1,
      "tauri:options": { application },
    },
  ],
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: { ui: "bdd", timeout: 180_000 },
  connectionRetryCount: 5,

  onPrepare() {
    fs.rmSync(dataDir, { recursive: true, force: true });
    for (const folder of ["data", "config", "cache", "appdata", "localappdata"]) {
      fs.mkdirSync(path.join(dataDir, folder), { recursive: true });
    }
    if (!fs.existsSync(application)) {
      throw new Error(`The app is not built: ${application}`);
    }
  },

  async beforeSession() {
    const driver = path.resolve(os.homedir(), ".cargo", "bin", `tauri-driver${exe}`);
    // On Windows, point EDGE_DRIVER at a msedgedriver that matches WebView2.
    const args = process.env.EDGE_DRIVER ? ["--native-driver", process.env.EDGE_DRIVER] : [];
    stopping = false;
    tauriDriver = spawn(driver, args, { stdio: [null, process.stdout, process.stderr] });
    tauriDriver.on("error", (error) => {
      console.error("tauri-driver could not start:", error);
      process.exit(1);
    });
    tauriDriver.on("exit", (code) => {
      if (!stopping) {
        console.error("tauri-driver exited with code", code);
        process.exit(1);
      }
    });
    await new Promise((resolve) => setTimeout(resolve, 2000));
  },

  afterSession() {
    stopDriver();
  },
};
