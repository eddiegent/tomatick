---
title: Configuration reference
sidebar_position: 3
---

# Configuration reference

Open the settings with the cog on the widget or from the tray menu. Press **Save** to apply your changes, or **Cancel** to leave them. A value outside its allowed range is refused with a message that tells you the range.

![The settings window](/img/screenshots/settings.png)

## Pomodoro cycle

| Setting | Default | Allowed |
|---|---|---|
| Work (minutes) | 25 | 1 to 180 |
| Short break (minutes) | 5 | 1 to 60 |
| Long break (minutes) | 15 | 1 to 120 |
| Work sessions before a long break | 4 | 2 to 12 |
| Daily goal (pomodoros) | 8 | 1 to 50 |
| Start the next phase automatically | Off | On or off |
| When the computer sleeps or locks | Pause the timer | Pause the timer, or keep it running |

**Reset to classic** loads 25, 5, 15 and 4 sessions into the form. You still have to save to apply them.

## Widget

The size of the widget: small (140 px), medium (200 px, the default) or large (280 px).

## Countdown timer

| Setting | Default | Allowed |
|---|---|---|
| Starting value | 10 minutes | 0:01 to 23:59:59 |
| Quick choices | 5, 10, 15, 30 minutes | One to six durations |

Type durations as minutes (`10`) or minutes and seconds (`1:30`). Separate the quick choices with commas. They appear when you click the time on the widget to edit the countdown.

## Alerts

A table lets you choose, for each of the four events, what happens.

![The alert settings](/img/screenshots/settings-alerts.png)

| Option | What it does |
|---|---|
| System notification | A native notification, with the exercise when there is one |
| Sound | Plays the sound chosen for this event |
| Full-screen overlay | Dims the screen with the phase, time left and exercise |
| Bring the widget to the front | Raises the widget above other windows |

The defaults are a notification and a sound for every event, and the overlay when a work phase ends and when a countdown ends.

Other options in this section:

- **Sound** for each event: a built-in sound (soft chime, bell, gentle beep, marimba) or **Add a file** to use your own MP3, WAV or OGG file. Defaults are the bell for the end of work, the chime for the end of a short break, the marimba for the end of a long break and the beep for the end of a countdown.
- **Volume**, from 0 to 100, for all sounds. It is independent of the system volume.
- **Mute all alert sounds.**
- **Test** buttons to listen to a sound.
- **Preview overlay** to see the overlay without touching the timer.

## Break exercises

An ordered list. For each entry, give a name and an optional amount (`30 s`, `15 reps`). Drag the handle to reorder, or use the move up and move down buttons, which also work with the keyboard. See [break exercises](./user-guide.md#break-exercises).

## System

| Setting | Default |
|---|---|
| Launch Tomatick when I log in | Off |
| Global shortcuts | `Ctrl+Alt+P` start or pause, `Ctrl+Alt+N` skip, `Ctrl+Alt+T` show or hide (`Cmd` on macOS) |
| Language | Same as the system |
| Check for new versions | On |

If a shortcut cannot be registered, or launch at login cannot be changed, the setting is saved and an error is shown in this section. Each shortcut has **Clear** and **Default** buttons.

## About

Shows the version, the license, a link to the source code and a link to this documentation. It also tells you whether a newer version exists, and **Check now** asks GitHub right away, even when the automatic check is off.
