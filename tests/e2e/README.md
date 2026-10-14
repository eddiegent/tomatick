# End-to-end tests

These tests drive the real app (the Rust core and the webview) through
[`tauri-driver`](https://v2.tauri.app/develop/tests/webdriver/) and
WebdriverIO. They cover what the mocked interface tests in `tests/ui` cannot:
the commands, the SQLite database, settings that survive a relaunch, the
history, and the shortcut registration.

Each file in `specs/` is one launch of the app, run in alphabetical order on
one scratch data folder, so a spec sees what the previous one saved. The
history spec takes about a minute because it lets a one-minute pomodoro run to
its end.

Not covered, because a WebDriver session cannot reach them: the tray icon,
global shortcuts pressed from another application, sleep, unplugging a
screen, launch at login, system notifications. Those stay manual.

## Run them

Linux and Windows only (there is no WebDriver for the macOS webview).

1. Build the app: `npm run tauri build -- --debug --no-bundle` (from the
   repository root). Set `TOMATICK_BIN` to test another build.
2. Install the driver: `cargo install tauri-driver --locked`.
3. A native WebDriver for the webview:
   - Linux: `sudo apt install webkit2gtk-driver xvfb`, then run the tests
     under `xvfb-run -a` if there is no display.
   - Windows: a `msedgedriver.exe` that matches the installed WebView2
     (version shown at `edge://version`). Put its path in `EDGE_DRIVER`.
4. `cd tests/e2e && npm install && npm test`.

The tests use a scratch data folder in the system temp folder. Your own
Tomatick data is not touched.
