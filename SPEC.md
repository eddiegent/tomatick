# Tomatick: Specification

Version: 0.1 (draft)
Date: 2026-10-11
License: MIT

## 1. Overview

Tomatick is a free, open source, cross-platform desktop timer built around the Pomodoro technique. Its identity is a small always-on-top tomato widget with a digital countdown in the middle. It also offers a simple one-shot countdown mode, break exercise suggestions, and local statistics.

The app is deliberately a pure timer. It has no task list, no projects and no accounts.

### 1.1 Goals

- A widget that is pleasant, ergonomic and simple to use.
- Fully customizable Pomodoro cycles, with the classic 25/5/15 profile as the default and a one-click reset to it.
- Breaks that are useful: an ordered exercise list rotates through short breaks and long break segments.
- Local history with clear charts (daily, weekly, monthly).
- Easy for outside contributors: clean architecture, translation-ready, documented.

### 1.2 Non-goals (version 1)

- Task lists, projects, tags or task-level statistics.
- Cloud sync, accounts, or any network feature other than the update check.
- Data export or import (the SQLite file stays internal).
- Mobile or web versions.
- Several simultaneous countdown timers.
- Automatic update installation.
- A "strict mode" overlay with no way out.
- Ticking sound during work phases.
- Recording countdown timers in history or statistics.

## 2. Platforms

| Platform | Support |
|---|---|
| Windows 10 and 11 | Yes |
| macOS (current and previous two major versions) | Yes |
| Linux (X11 and Wayland) | Yes, with Wayland limitations (see 15) |

## 3. Technology stack

| Layer | Choice | Notes |
|---|---|---|
| App shell | Tauri (current stable major) | Native installers, small footprint, system webview |
| Core | Rust | Timer engine, SQLite, audio, notifications, tray, shortcuts, power events, update check |
| UI | React with TypeScript, built with Vite | Widget, settings dialog, stats window, overlay |
| Storage | SQLite (single local file in the app data directory) | History, exercise list and settings |
| i18n | i18next with react-i18next | English and French at launch |
| Charts | A React chart library (Recharts suggested) | Bar charts and simple distributions |
| Audio | Rust audio crate (rodio suggested) | Alerts work independently of webview state |
| Docs site | Docusaurus, in `/website` | Deployed to GitHub Pages by GitHub Actions |
| CI | GitHub Actions | Build matrix for the three platforms, tests, docs deploy |

The user is not required to use a browser: the UI runs inside the operating system webview embedded in the app window (WebView2 on Windows, WebKit on macOS, WebKitGTK on Linux).

## 4. Modes

The app has two modes, and only one is active at a time. The two modes never run simultaneously.

- **Pomodoro mode:** cycles of work and breaks (section 5).
- **Countdown mode:** one timer, set to a chosen duration, that alerts once at zero (section 6).

Switching mode is done from a control on the widget (visible on hover) and from the tray menu.

**Reset on switch:** switching mode resets the timer of the mode being left.

- Leaving Pomodoro mode resets the cycle to the start of a work phase. If the phase had started, it is recorded in the history as `interrupted`.
- Leaving Countdown mode returns the countdown to its set duration, not started.
- The settings (durations, presets, exercise list) are untouched. Only the running state is reset.

## 5. Pomodoro mode

### 5.1 Cycle and defaults

Phases are `work`, `short_break` and `long_break`. A cycle is N work phases, each followed by a short break, except that after the Nth work phase the break is a long break. Then the counter resets.

Classic defaults (the "classic profile"):

| Setting | Default |
|---|---|
| Work duration | 25 min |
| Short break duration | 5 min |
| Long break duration | 15 min |
| Work phases before a long break | 4 |
| Auto-start next phase | Off |
| Daily goal (pomodoros) | 8 |

All values are editable in the configuration dialog. A "Reset to classic defaults" button restores the classic profile in one click (with a confirmation, since it overwrites the user's values).

Suggested validation ranges (adjust after testing): work 1 to 180 min, short break 1 to 60 min, long break 1 to 120 min, work phases before a long break 2 to 12.

### 5.2 Controls and behavior

| Action | Behavior |
|---|---|
| Start | Starts the current phase, or the next one if the timer is waiting |
| Pause | Holds the timer until the user resumes (use case: lunch, a call). No time limit |
| Resume | Continues from the held time |
| Skip | Ends the current phase now and moves to the next one |
| Reset | Returns the current phase to its full duration, not started |
| Extend (breaks only) | Adds 1 or 5 minutes to the current break, from the overlay |

Auto-start is a single toggle. When it is on, the next phase starts automatically when a phase ends. When it is off, the timer waits for the user to press start.

Skipping a work phase does not count as a completed pomodoro (it is recorded as `skipped`), but it does advance the position in the cycle, so skipping repeatedly still reaches the long break (decided, see 20).

### 5.3 Engine state machine

The engine lives in Rust and is the single source of truth. The UI only renders state and sends commands.

| State | Meaning |
|---|---|
| `Idle` | Nothing running, waiting for the user to start the current phase |
| `Running` | A phase is counting down |
| `Paused` | Paused manually by the user |
| `AutoPaused` | Paused by the system (sleep or lock), see 5.4 |

Rules:

- Remaining time is computed from a monotonic clock and wall-clock timestamps, never by counting UI ticks, so it stays accurate if the UI is throttled.
- The engine takes an injectable clock so cycle logic can be unit tested without waiting.
- The engine emits a state snapshot event to all windows (mode, phase kind, remaining time, running or paused state, cycle position, current label and segment). It emits at least once per second while running.
- The engine accepts commands: `start`, `pause`, `resume`, `skip`, `reset`, `extend(seconds)`, `set_mode`, `set_countdown(duration)`.
- The engine runs one timer at a time. `set_mode` resets the timer of the mode being left before activating the other one (see section 4), so no timer ticks, records history or fires alerts in an inactive mode.
- Leaving Pomodoro mode also resets the cycle counter (the position within the N work phases).

### 5.4 Sleep, lock and absence

A setting chooses the behavior when the computer sleeps or the screen locks:

- **Auto-pause (default):** the timer pauses on sleep or lock and enters `AutoPaused`. On wake or unlock, it stays paused until the user resumes.
- **Keep running:** the timer follows the real clock. A phase may end while the user is away, and its alerts fire on wake at the latest.

The time spent paused is recorded in the history (`paused_seconds`) so statistics stay honest.

Power and lock events use platform listeners behind one Rust trait (Windows session and power notifications, macOS workspace notifications, Linux logind over D-Bus).

### 5.5 Crash and quit handling

If the app quits or crashes during a phase, that phase is recorded as `interrupted` the next time the app starts. The app does not restore a half-finished phase.

## 6. Countdown mode

- One countdown at a time.
- The user sets the duration by clicking the digital display and typing a time (for example `10:00`, up to `23:59:59`), or by choosing a preset.
- Presets default to 5, 10, 15 and 30 minutes. One to six of them can be kept, and they are editable in the configuration dialog.
- Controls: start, pause, resume, reset. Skip does not apply.
- At zero, the app fires the "countdown end" alert (section 8).
- Countdowns are not recorded in history or statistics.
- Switching to Pomodoro mode resets the countdown to its set duration (see section 4).
- Visual identity: same tomato and progress ring, in amber, so the modes are told apart at a glance.

## 7. Break exercises

### 7.1 Exercise list

The user maintains one ordered list in the configuration dialog (examples: planks, sit-ups, stretching).

- Each entry has a name and an optional duration or repetition count (free text such as `30 s` or `15 reps`).
- The list can be reordered by drag and drop (with keyboard-accessible move up and down controls).
- The list is shared by short breaks and long breaks.
- The app rotates through the list in order and wraps around at the end. The rotation position persists across sessions.
- If the list is empty, breaks have no label.

### 7.2 Short breaks

A short break shows the next exercise in the rotation as its label.

### 7.3 Long breaks (automatic segments)

A long break is divided into short-break-sized segments, and each segment shows the next exercise in the rotation.

- Number of segments = `floor(long break duration / short break duration)`, with a minimum of 1.
- Any remainder is added to the last segment.
- Example: long break 20 min, short break 5 min gives 4 segments of 5 min and 4 consecutive exercises. Long break 17 min gives 3 segments (5, 5, 7 min).
- The rotation carries on into the next break.
- Moving from one segment to the next updates the label with a light cue (the break-end sound, no overlay and no system notification). Decided and implemented, see 20.

### 7.4 Where the label appears

- On the widget during the break (medium and large sizes; in the tooltip on the small size).
- In the break overlay.
- In the system notification that announces the break.

## 8. Alerts

### 8.1 Events

| Event | Fired when |
|---|---|
| `work_end` | A work phase ends |
| `short_break_end` | A short break ends |
| `long_break_end` | A long break ends |
| `countdown_end` | The countdown reaches zero |

### 8.2 Alert types

Each event has its own independent configuration for each type:

| Type | Behavior |
|---|---|
| System notification | Native OS notification, with the break label when relevant |
| Sound | Built-in sound or a user-supplied file (see 8.3) |
| Full-screen overlay | See 8.4 |
| Bring window to front | Raises and focuses the widget window |

### 8.3 Sounds

- A small set of built-in sounds ships with the app (for example a soft chime, a bell, a gentle beep). One can be chosen per event.
- The user can add their own audio file (MP3, WAV or OGG) for any event. Files are copied into the app data directory so they keep working if the original moves.
- A volume slider (independent of the system volume) and a mute toggle apply to all alerts.
- A "Test" button lets the user preview each sound.

### 8.4 Full-screen overlay

For break starts (and for the countdown end), the overlay is a dimmed full-screen layer showing the phase, the remaining time and the exercise label.

Break overlay actions:

- **Dismiss:** close it (Escape or a button). The break keeps running on the widget.
- **Skip break:** ends the break early and starts the next work phase.
- **Extend:** "+1 min" and "+5 min" buttons. Added time counts in the history (`extended_seconds`).

Countdown overlay: a "Time's up" message with **Dismiss** and **Restart**. No skip or extend.

There is no strict mode. By default the overlay covers the display that holds the widget (decided, see 20).

## 9. Widget

### 9.1 Window behavior

- Frameless, transparent background, always on top, no taskbar entry (the tray icon is the fallback).
- Draggable by the tomato body.
- Position and monitor are remembered and restored on launch (always on, not a setting). If the saved monitor is gone, the widget falls back to the primary display.
- Three sizes selectable in settings: small, medium, large (140, 200 and 280 px).

### 9.2 Appearance

- A tomato drawn in SVG, with a digital display in the center showing `mm:ss` (and `h:mm:ss` when an hour or more is shown).
- Color by state:

| State | Color |
|---|---|
| Work | Red |
| Short break | Green |
| Long break | Blue |
| Countdown | Amber |

- A progress ring around the tomato shows progress through the current phase (or segment, during a long break).
- Paused state is clearly visible: the tomato is dimmed and a pause symbol shows over the countdown. The same treatment applies to `AutoPaused`, with a distinct small indicator (for example a moon) so the user knows it was automatic.
- The cycle position (for example `2/4`) is shown subtly.
- Animations respect the system "reduce motion" setting.

### 9.3 Controls

The countdown is always visible. On hover, symbol buttons fade in around the tomato so the small size stays clean:

| Control | Action |
|---|---|
| Play / Pause | Start, pause, resume |
| Skip | Next phase (Pomodoro mode) |
| Reset | Reset current phase |
| Mode toggle | Switch between Pomodoro and Countdown |
| Chart icon | Open the statistics window |
| Cog | Open the configuration dialog |

Clicking the digital display in Countdown mode lets the user type a duration. The goal is an ergonomic, simple-to-use widget: no control should need more than one click.

## 10. Configuration dialog

A separate window, with sections:

1. **Timer:** durations, work phases before a long break, auto-start toggle, daily goal, sleep and lock behavior, Reset to classic defaults.
2. **Countdown:** the editable presets.
3. **Alerts:** a matrix of the four events by the four alert types, plus sound selection, custom file picker, volume and mute, test buttons.
4. **Exercises:** the ordered list with add, edit, delete and drag-and-drop reordering.
5. **Appearance:** widget size.
6. **System:** launch at login, global shortcuts, language, update check on or off.
7. **About:** version, license, link to the repository and documentation.

Changes apply immediately (no separate save step), except the destructive Reset which asks for confirmation.

## 11. Statistics window

A separate window opened from the chart icon. Pomodoro data only.

- **Today at a glance:** completed pomodoros, total focus time, progress toward the daily goal.
- **Day, week and month views:** bar charts of completed pomodoros and focus time, with previous and next navigation.
- **Streak:** consecutive days with at least one completed pomodoro (current and longest).
- **Completion rate:** completed work phases compared with skipped or interrupted ones.
- **Best time of day:** hourly distribution of focus sessions.
- **Totals and averages:** per period, plus average focus time per active day.

Definitions:

- A pomodoro is a completed work phase.
- Focus time is the sum of `actual_seconds` of work phases, excluding paused time.
- Days are grouped by the user's local time zone, using UTC timestamps in storage.
- The first day of the week follows the system locale.

## 12. Data model (SQLite)

One database file in the OS-standard app data directory. Migrations are versioned and run at startup.

```sql
CREATE TABLE phases (
  id               INTEGER PRIMARY KEY,
  kind             TEXT NOT NULL CHECK (kind IN ('work','short_break','long_break')),
  started_at       TEXT NOT NULL,              -- UTC, ISO 8601
  ended_at         TEXT,
  planned_seconds  INTEGER NOT NULL,
  actual_seconds   INTEGER NOT NULL DEFAULT 0,
  paused_seconds   INTEGER NOT NULL DEFAULT 0,
  extended_seconds INTEGER NOT NULL DEFAULT 0,
  status           TEXT NOT NULL CHECK (status IN ('completed','skipped','interrupted')),
  labels           TEXT                        -- JSON array of exercise names shown
);

CREATE TABLE exercises (
  id       INTEGER PRIMARY KEY,
  position INTEGER NOT NULL,
  name     TEXT NOT NULL,
  amount   TEXT                                 -- free text: "30 s", "15 reps"
);

CREATE TABLE settings (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL                           -- JSON-encoded
);
```

- A long break is stored as one row, with all its segment labels in `labels`.
- Status meanings: `completed` ran to its end, `skipped` was ended by the user, `interrupted` was abandoned (reset after start, mode switch, quit or crash).
- Indexes on `started_at` and `(kind, status)`.
- Statistics are computed with grouped queries from `phases`. No pre-aggregated tables in version 1.
- No export or import. History is kept indefinitely (the volume is tiny).

## 13. Desktop integration

### 13.1 Tray icon

Shows or hides the widget, starts or pauses the timer, switches mode, opens settings and statistics, and quits. Tray text is translated.

### 13.2 Launch at login

An option in the System section.

### 13.3 Global shortcuts

Work even when another application has focus, and can be rebound in the configuration dialog:

| Action | Suggested default |
|---|---|
| Start / pause | `Ctrl+Alt+P` (`Cmd+Alt+P` on macOS) |
| Skip to next phase | `Ctrl+Alt+N` (`Cmd+Alt+N` on macOS) |
| Show / hide widget | `Ctrl+Alt+T` (`Cmd+Alt+T` on macOS) |

The settings UI detects conflicts with an already registered shortcut and reports registration failures instead of failing silently.

### 13.4 Single instance

Only one copy of Tomatick runs per user. Starting it again (the shortcut, a second login entry) does not open a second timer, tray icon or set of global shortcuts: the running copy shows its widget and the new one exits.

## 14. Internationalization

- All user-visible text goes through i18next. No hard-coded strings.
- Languages at launch: English and French. The default follows the system language, and it can be overridden in settings.
- Locale files are plain JSON (`locales/en.json`, `locales/fr.json`) so a contributor can add a language with one file and no code change.
- Text produced on the Rust side (tray menu, notifications) uses the same locale files.
- Dates, times and numbers are formatted with the active locale.

## 15. Platform notes and known limits

- **Linux and Wayland:** always-on-top, global shortcuts and window positioning depend on the compositor and are not reliable under Wayland. They work on X11. The app must degrade gracefully (for example fall back to the tray icon and in-app shortcuts) and the documentation must list these limits.
- **Webview dependency:** Windows 11 includes WebView2, and the Windows installer fetches it on Windows 10 if missing. On Linux, WebKitGTK is a package dependency.
- **Rendering differences:** each platform uses a different engine, so the tomato, transparency and charts need testing on all three.
- **Unsigned builds:** without code-signing certificates, Windows SmartScreen and macOS Gatekeeper show warnings on first launch. The install documentation must explain how to proceed. Signing can be added later.

## 16. Distribution, updates, repository and docs

### 16.1 Distribution

Installers are built by GitHub Actions on release tags and published on GitHub Releases: MSI or EXE for Windows, DMG for macOS, AppImage and `.deb` for Linux.

### 16.2 Update check (check only)

- On launch and then periodically (for example every 24 hours), the app queries the GitHub Releases API for the latest version.
- If a newer version exists, it shows a discreet notice with a link to the releases page. It never downloads or installs anything itself.
- Can be turned off in settings. This is the only network request the app makes.

### 16.3 License

MIT. The `LICENSE` file is at the repository root.

### 16.4 Repository layout

```
tomatick/
  src-tauri/          Rust core (engine, db, audio, alerts, power, shortcuts, tray, updater)
  src/                React UI (widget, settings, stats, overlay)
  src/locales/        Translation files
  website/            Docusaurus documentation site
  .github/workflows/  CI, release builds, docs deploy
  LICENSE
  README.md
  SPEC.md
```

### 16.5 Documentation site

Docusaurus in `/website`, deployed to GitHub Pages by a GitHub Actions workflow, with English and French content. Suggested pages: installation (including the unsigned-build and Wayland notes), user guide, configuration reference, contributing guide, architecture overview, translation guide, FAQ.

## 17. Proposed Rust module layout

| Module | Responsibility |
|---|---|
| `engine` | Pure state machine and cycle logic, injectable clock, fully unit tested |
| `db` | SQLite access, migrations, stats queries |
| `settings` | Typed settings with defaults and validation |
| `audio` | Playback, volume, built-in and custom sounds |
| `alerts` | Maps engine events to notification, sound, overlay, bring-to-front |
| `power` | Sleep, wake, lock and unlock listeners per platform |
| `shortcuts` | Global shortcut registration and rebinding |
| `tray` | Tray icon and menu |
| `updater` | Release check |
| `commands` | Tauri IPC commands and event emission |

## 18. Accessibility and quality

- Every control has an accessible label and is reachable by keyboard in the dialogs and the overlay.
- Digit contrast meets WCAG AA on all four phase colors.
- Reduced-motion setting is respected.
- Tests: unit tests for the engine and stats queries, and a CI build on all three platforms for every pull request.

## 19. Suggested build milestones

1. **Skeleton:** Tauri and React project, CI building on three platforms, MIT license, README.
2. **Engine:** state machine, classic profile, unit tests, command and event plumbing.
3. **Widget:** tomato, colors, ring, digital display, hover controls, always-on-top, position memory, three sizes.
4. **Settings and persistence:** SQLite, migrations, configuration dialog, reset to classic defaults.
5. **Alerts:** notification, sound, bring to front, then the overlay with dismiss, skip and extend.
6. **Break exercises:** list, rotation, long break segments, labels in widget, overlay and notifications.
7. **Countdown mode:** mode toggle, duration entry, presets, amber theme, countdown alert.
8. **Desktop integration:** tray, launch at login, global shortcuts, sleep and lock handling.
9. **History and statistics:** phase recording, stats window and charts.
10. **i18n:** i18next wiring, English and French files, Rust-side translations.
11. **Update check, packaging and docs:** release workflow, installers, Docusaurus site, Linux and Wayland testing, 0.1 release.

## 20. Decisions

These points were not part of the first design discussion. They were proposed as assumptions and have all been confirmed.

- **Overlay display:** covers the display that holds the widget, not all displays (section 8.4).
- **Global shortcut defaults:** the key combinations in 13.3.
- **Settings storage:** in the same SQLite file as the history (section 12).
- **Extend buttons:** +1 min and +5 min (section 8.4).
- **Sound extras:** volume slider and mute toggle are included, ticking is excluded (section 8.3).
- **Project name:** Tomatick. Availability checks are summarized in section 21. The domain names have not been checked.

Decided: only one mode is active at a time, and switching mode resets the timer of the mode being left (section 4).

Decided: a skipped work phase is not counted as a pomodoro but advances the position in the cycle, whether or not it had started (section 5.2).

Decided and implemented: a move to the next segment of a long break plays the break-end sound, with no overlay and no system notification (section 7.3).

Decided and implemented: the daily goal defaults to 8 pomodoros (section 5.1).

Decided and implemented: the widget sizes are 140, 200 and 280 px (section 9.1).

## 21. Name availability check (2026-10-11)

| Place | Result |
|---|---|
| npm package `tomatick` | Not found, appears free |
| crates.io crate `tomatick` | Not found, appears free |
| GitHub user or organization `tomatick` | Taken by an unrelated personal account (portfolio repositories). The repository can live under your own account or an organization with another name |
| Domains (.com, .app, .dev, .io, .org) | Not verified, the check could not run from the build environment. Check with a registrar |
| Similar projects | A Rust Pomodoro crate named `tomat` exists (a CLI with a daemon). The name is close but distinct, and `tomatick` is not used by a timer project that came up in a web search |
