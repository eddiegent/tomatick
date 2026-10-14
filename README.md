# Tomatick

A free, open source Pomodoro timer for Windows, macOS and Linux, built around a small always-on-top tomato widget.

> **Status: first release in preparation.** Every feature of the first version is in place. The installers are published on the [releases page](https://github.com/eddiegent/tomatick/releases) once version 0.1.0 is tagged. The builds are not signed, so Windows and macOS warn on first launch. The [documentation](https://eddiegent.github.io/tomatick/) explains how to continue.

## What it is

Tomatick is a pure timer. No task list, no accounts, no cloud. A tomato sits on your desktop with a digital countdown in the middle, and it gets out of your way.

## Features

- **Tomato widget:** always on top, three sizes (small, medium, large), color by phase (red for work, green for short break, blue for long break), a progress ring, and a clearly visible paused state. Controls appear when you hover over it.
- **Fully customizable Pomodoro cycles:** the classic 25/5/15 profile is the default, with a one-click reset back to it. Auto-start of the next phase is a toggle, and you can pause at any time (for lunch, a call).
- **Countdown mode:** a simple one-shot timer for tea, pasta or a quick task. One timer at a time, with editable presets.
- **Break exercises:** keep an ordered list (planks, sit-ups, stretches) and the app rotates through it. A long break is split into short-break-sized segments, each with its own exercise.
- **Configurable alerts:** system notification, sound, full-screen overlay and bring-to-front, set separately for each event. Built-in sounds or your own audio files.
- **History and statistics:** daily, weekly and monthly charts, streaks, completion rate and best time of day, all stored locally in SQLite.
- **Desktop integration:** tray icon, launch at login, global keyboard shortcuts, and pausing when the computer sleeps.
- **Translation-ready:** English and French at launch, with simple JSON files for adding more languages.

## Platforms

| Platform | Installer |
|---|---|
| Windows 10 and 11 | `.exe` |
| macOS (Intel and Apple silicon) | `.dmg` |
| Linux (X11) | AppImage and `.deb` |
| Linux (Wayland) | AppImage and `.deb`, with limits: always-on-top and global shortcuts depend on the compositor |

## Tech stack

- [Tauri](https://tauri.app) with a Rust core (timer engine, SQLite, notifications, tray, shortcuts)
- React with TypeScript for the UI
- SQLite for local storage
- i18next for translations
- Docusaurus for the documentation site, hosted on GitHub Pages

The app runs in the operating system's built-in webview. No browser is needed.

## Documentation

The documentation is at **[eddiegent.github.io/tomatick](https://eddiegent.github.io/tomatick/)**, in English and French: installation, user guide, configuration reference, FAQ, and guides for contributors (architecture and translation). The source is in the `website` folder.

The full specification is in [SPEC.md](SPEC.md). It covers the timer behavior, alerts, widget design, statistics, data model, architecture and build milestones.

The images of the site are rendered from the interface with sample data, and are meant to be replaced with real captures: see `website/static/img/screenshots/README.md`, which also covers the French set.

## Development

Prerequisites:

- [Node.js](https://nodejs.org) 22 or later
- [Rust](https://rustup.rs) (stable)
- The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your platform (on Linux this includes WebKitGTK 4.1)

Commands:

```
npm install            # install frontend dependencies
npm run tauri dev      # run the app in development mode
npm run build          # type-check and build the frontend
npm run tauri build    # build installers for your platform
```

The timer engine (`src-tauri/src/engine`) is plain Rust with no dependency on Tauri, and its unit tests run with:

```
cargo test --manifest-path src-tauri/Cargo.toml
```

### Trying the app

`npm run tauri dev` opens the tomato: a frameless, transparent, always-on-top window. Drag it by its body and hover over it to reveal the controls. The [user guide](https://eddiegent.github.io/tomatick/docs/user-guide) describes them all. Settings and history are stored in a SQLite file in the application data folder, and the tray menu has **Quit**.

Tests:

```
npm run test:ui        # the interface, against a fake backend (Playwright)
```

The end-to-end tests of the real app are described in `tests/e2e/README.md`. The documentation site is built with `cd website && npm install && npm run build`.

Continuous integration builds the app on Windows, macOS and Linux for every push to `main` and every pull request. Pushing a tag such as `v0.1.0` builds the installers and prepares a draft release.

## Contributing

Bug reports, ideas, translations and code are welcome. For anything bigger than a small fix, open an issue first. The [contributing guide](https://eddiegent.github.io/tomatick/docs/contributing) explains how to run and test the project, and the [translation guide](https://eddiegent.github.io/tomatick/docs/translation) shows how to add a language with one JSON file.

## License

[MIT](LICENSE)
