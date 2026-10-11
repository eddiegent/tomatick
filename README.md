# Tomatick

A free, open source Pomodoro timer for Windows, macOS and Linux, built around a small always-on-top tomato widget.

> **Status: early development.** There is no released version yet. The design is written down in [SPEC.md](SPEC.md), and the project skeleton (Tauri, React, CI) is in place. The widget and timer engine are next.

## What it is

Tomatick is a pure timer. No task list, no accounts, no cloud. A tomato sits on your desktop with a digital countdown in the middle, and it gets out of your way.

## Planned features

- **Tomato widget:** always on top, three sizes (small, medium, large), color by phase (red for work, green for short break, blue for long break), a progress ring, and a clearly visible paused state. Controls appear when you hover over it.
- **Fully customizable Pomodoro cycles:** the classic 25/5/15 profile is the default, with a one-click reset back to it. Auto-start of the next phase is a toggle, and you can pause at any time (for lunch, a call).
- **Countdown mode:** a simple one-shot timer for tea, pasta or a quick task. One timer at a time, with editable presets.
- **Break exercises:** keep an ordered list (planks, sit-ups, stretches) and the app rotates through it. A long break is split into short-break-sized segments, each with its own exercise.
- **Configurable alerts:** system notification, sound, full-screen overlay and bring-to-front, set separately for each event. Built-in sounds or your own audio files.
- **History and statistics:** daily, weekly and monthly charts, streaks, completion rate and best time of day, all stored locally in SQLite.
- **Desktop integration:** tray icon, launch at login, global keyboard shortcuts, and sleep and lock handling.
- **Translation-ready:** English and French at launch, with simple JSON files for adding more languages.

## Platforms

| Platform | Status |
|---|---|
| Windows 10 and 11 | Planned |
| macOS | Planned |
| Linux (X11) | Planned |
| Linux (Wayland) | Planned, with limits: always-on-top and global shortcuts depend on the compositor |

## Tech stack

- [Tauri](https://tauri.app) with a Rust core (timer engine, SQLite, audio, notifications, tray, shortcuts)
- React with TypeScript for the UI
- SQLite for local storage
- i18next for translations
- Docusaurus for the documentation site, hosted on GitHub Pages

The app runs in the operating system's built-in webview. No browser is needed.

## Documentation

The full specification is in [SPEC.md](SPEC.md). It covers the timer behavior, alerts, widget design, statistics, data model, architecture and build milestones.

A documentation website is planned in the `website` folder and will be published with GitHub Pages.

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

Continuous integration builds the app on Windows, macOS and Linux for every push to `main` and every pull request.

## Contributing

The app does not do anything useful yet, so code contributions are premature, but feedback on the specification is welcome. Open an issue to discuss a change or a question about the design.

Once the timer engine and widget land, this section will describe how to add a translation.

## License

[MIT](LICENSE)
