---
title: Contributing
sidebar_position: 5
---

# Contributing

Tomatick is open source under the [MIT license](https://github.com/eddiegent/tomatick/blob/main/LICENSE). Bug reports, ideas, translations and code are all welcome. For a change that is more than a small fix, please open an issue first to discuss it.

The design is written down in [SPEC.md](https://github.com/eddiegent/tomatick/blob/main/SPEC.md). It explains what the app does and why, and it says what is out of scope (no task list, no accounts, no cloud). Please read it before you propose a feature.

## What you need

- [Node.js](https://nodejs.org) 22 or later
- [Rust](https://rustup.rs) (stable)
- The [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your system. On Linux this includes WebKitGTK 4.1.

## Run it

```bash
npm install            # install the frontend dependencies
npm run tauri dev      # run the app in development mode
npm run build          # type-check and build the frontend
npm run tauri build    # build installers for your system
```

## Test it

```bash
cargo test --manifest-path src-tauri/Cargo.toml   # the Rust core
npm run test:ui                                   # the interface, with a fake backend
```

The engine (`src-tauri/src/engine`) is plain Rust and has no dependency on Tauri, so its tests are fast. The interface tests use Playwright with a stand-in for the Tauri backend (`tests/ui/mock.ts`), so they run in a plain browser.

The end-to-end tests drive the real app through `tauri-driver` and WebdriverIO. They cover the commands, the database and the settings that survive a restart. See `tests/e2e/README.md` for how to run them, on Linux and Windows. Some things cannot be automated and stay manual: the tray icon, global shortcuts pressed from another application, sleep, unplugging a screen, launch at login, and system notifications.

Continuous integration builds the app on Windows, macOS and Linux for every pull request, and runs the checks. Please run `cargo fmt` and make sure `cargo clippy` is quiet before you push.

## Documentation

This site is in the `website` folder and uses [Docusaurus](https://docusaurus.io).

```bash
cd website
npm install
npm start                  # English, with live reload
npm start -- --locale fr   # French
npm run build              # build both languages
```

The English pages are in `website/docs`, and the French ones in `website/i18n/fr/docusaurus-plugin-content-docs/current`. When you change a page, change both if you can, and say so in your pull request if you can only do one.

The images are in `website/static/img/screenshots`. They are rendered from the interface with sample data by `npm run docs:screenshots`, run from the root of the repository. To replace one with a real capture, save it under the same name.

## Pull requests

- Keep a pull request to one subject.
- Add or update tests when you change behavior.
- Text in the interface goes through the translation files. Do not hard-code strings, see the [translation guide](./translation.md).
- Changes to the design go in SPEC.md as well.
