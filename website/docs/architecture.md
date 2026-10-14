---
title: Architecture
sidebar_position: 6
---

# Architecture

Tomatick is a [Tauri 2](https://tauri.app) application. A Rust core does the work and a React interface, written in TypeScript, draws it in the webview of the operating system. No browser is bundled.

```text
tomatick/
  src-tauri/          Rust core
  src/                React interface
  locales/            Translation files (shared by both sides)
  tests/              Interface tests, end-to-end tests, documentation images
  website/            This site
  .github/workflows/  CI, release builds, docs deploy
```

## The rule

**The engine in Rust is the single source of truth.** The interface only draws the state it receives and sends commands. Nothing in the interface counts time.

## The Rust core

| Module | Responsibility |
|---|---|
| `engine` | The state machine and the cycle logic. Pure Rust, with an injectable clock so the tests do not have to wait |
| `storage` | SQLite: migrations, settings, exercises and their rotation |
| `history` | Recording phases and the statistics queries |
| `alerts` | Maps engine events to a notification, a sound, the overlay and bring-to-front |
| `overlay` | The full-screen overlay windows |
| `placement` | Remembering the position of the widget and the screen it is on |
| `power` | Noticing that the computer slept |
| `shortcuts` | Global shortcut validation and registration |
| `tray` | The tray icon and its menu |
| `updater` | The check for a newer release |
| `i18n` | Translations for the tray menu, notifications and window titles |
| `lib.rs` | The Tauri commands, the events and the ticker that drives everything |

The engine has four states: idle, running, paused by the user, and paused by the system. It computes the time left from a monotonic clock and timestamps, never by counting ticks, so the time stays right if the interface is throttled or the computer is busy.

A ticker thread calls the engine every 250 ms. It turns what the engine reports into history rows, alerts and events, and it notices when the computer slept.

## Talking to the interface

The interface calls Tauri commands (`invoke`) and listens for events.

| Event | When |
|---|---|
| `engine:state` | After every command, and while a timer runs. The payload is a snapshot: mode, phase, time left, segment, label, cycle position |
| `engine:event` | A phase was recorded, an alert is due, or a long break moved to its next part |
| `settings:changed` | The settings were saved |
| `history:changed` | A phase was added to the history |
| `update:changed` | An update check ended |

## The interface

One bundle serves every window. The window label decides what is drawn.

| Folder | Window |
|---|---|
| `src/widget` | The tomato |
| `src/settings` | The settings |
| `src/stats` | The statistics |
| `src/overlay` | The full-screen overlay |
| `src/i18n` | i18next set-up, shared by all of them |

## Data

One SQLite file, `tomatick.db`, in the application data folder. Migrations are versioned and run at startup. Settings are stored as JSON values in a `settings` table, the exercise list in `exercises`, and every phase in `phases`, with its status (completed, skipped or interrupted) and the time paused and extended. Timestamps are UTC, and days are grouped by the local time zone when the statistics are computed. Statistics come from grouped queries, with no pre-computed tables.

## Translations

The files in `locales` are read by the interface at build time and compiled into the Rust core by `build.rs`, so the tray menu and the notifications use the same text as the windows.

## Network

The only network request is the update check: `updater` asks the GitHub Releases API for the latest release and compares versions. It never downloads anything.

## Tests

- Unit tests in Rust, for the engine, the storage and the statistics.
- Playwright tests of the interface against a fake backend (`tests/ui`).
- End-to-end tests of the real app with `tauri-driver` (`tests/e2e`).
- A CI build on Windows, macOS and Linux for every pull request.

## Releases

Pushing a tag that starts with `v` runs the release workflow. It checks that the tag matches the version in the project files, builds the installers on the three systems, and creates a draft release on GitHub for the maintainer to review and publish.
