---
title: User guide
sidebar_position: 2
---

# User guide

Tomatick is a pure timer. There is no task list and no account. This page covers everything you can do with it.

:::note About the images
The images in this documentation were rendered from the interface with sample data. They are not captures of a real desktop, and they will be replaced.
:::

## The widget

The widget is a tomato with the time left in the middle. It has no window frame, stays on top of your other windows, and you can drag it by its body. Tomatick remembers where you put it, including the screen. If that screen is gone the next time you start, the widget moves to the main screen.

![The widget during a work phase](/img/screenshots/widget-work.png)

The color tells you what is going on, and a ring around the tomato shows how far you are into the phase.

| Color | Meaning |
|---|---|
| Red | Work |
| Green | Short break |
| Blue | Long break |
| Amber | Countdown |

![Short break](/img/screenshots/widget-short-break.png)

The small number above the time, such as `2/4`, is the position in the cycle. Pick a size in Settings: small (140 px), medium (200 px) or large (280 px). On the medium and large sizes, the exercise of a break is shown under the time. On the small size it is in the tooltip.

### Buttons

Move the mouse over the widget and the buttons fade in around the tomato.

![The widget with its buttons](/img/screenshots/widget-controls.png)

| Button | What it does |
|---|---|
| Play and pause | Starts the phase, pauses it, or resumes it |
| Skip | Ends the phase now and goes to the next one (Pomodoro mode only) |
| Reset | Puts the current phase back to its full length, not started |
| Mode | Switches between Pomodoro and Countdown |
| Chart | Opens the statistics |
| Cog | Opens the settings |

A paused timer is dimmed and shows a pause symbol. If the timer was paused by the computer going to sleep, a small moon is shown as well, so you know you did not do it yourself.

![A paused timer](/img/screenshots/widget-paused.png)

## Pomodoro mode

A cycle is a number of work phases, each followed by a short break, except the last one, which is followed by a long break. The default is the classic one: 25 minutes of work, 5 minutes of short break, 15 minutes of long break, a long break after 4 work phases. Settings has a **Reset to classic** button that loads these values again.

- **Pause** holds the timer for as long as you want, for lunch or a call.
- **Auto-start** is off by default. When it is off, the timer waits for you to press play after each phase. When it is on, the next phase starts by itself.
- **Skip** on a work phase does not count it as a completed pomodoro, but it does move you forward in the cycle, so skipping again and again still gets you to the long break.
- **Reset** on a phase you had started is recorded in the history as interrupted.
- If the app quits or crashes during a phase, that phase is recorded as interrupted the next time you start. A half-finished phase is not restored.

### Break exercises

Keep a list of exercises in Settings (planks, sit-ups, stretches), each with an optional amount such as `30 s` or `15 reps`. Every break shows the next exercise in the list, and the list starts over when it ends. The position is remembered between sessions. With an empty list, breaks have no label.

A long break is split into several parts, as long as a short break, and each part has its own exercise. A 20 minute long break with 5 minute short breaks gives 4 parts and 4 exercises in a row. If the long break is not a multiple of the short break, the rest goes to the last part. A sound plays when the break moves on to the next part.

![The long break](/img/screenshots/widget-long-break.png)

## Countdown mode

Countdown mode is a simple timer for tea, pasta or a quick task. There is one timer at a time, it is shown in amber, and it alerts once at zero. Countdowns are not recorded in the statistics.

![The countdown](/img/screenshots/widget-countdown.png)

When the timer is not running, click the time to type a new duration. Type minutes (`10`) or minutes and seconds (`1:30`) and press Enter. Quick choices are shown too, `5`, `10`, `15` and `30` minutes by default, and you can change them in Settings.

Only one mode is active at a time. Switching mode resets the timer you leave: a Pomodoro phase that had started is recorded as interrupted, and the countdown goes back to its set duration. Your settings are not touched.

## Alerts

For each of four events, you choose what happens: a system notification, a sound, a full-screen overlay, and bringing the widget to the front.

| Event | When |
|---|---|
| Work ends | A break starts |
| Short break ends | Time to work |
| Long break ends | Time to work |
| Countdown ends | The countdown reaches zero |

### The full-screen overlay

The overlay dims the screen that holds the widget and shows the phase, the time left and the exercise.

![The break overlay](/img/screenshots/overlay-break.png)

On a break you can:

- **Dismiss** it (Escape or the button). The break keeps running on the widget.
- **Skip break**, which ends the break and starts the next work phase.
- **Extend** the break by 1 or 5 minutes. The extra time is kept in the history.

At the end of a countdown the overlay says "Time's up", with **Dismiss** and **Restart**.

There is no strict mode: the overlay never traps you. Settings has a **Preview overlay** button to see it with sample content, without touching the timer.

### Sounds

Four sounds come with the app: a soft chime, a bell, a gentle beep and a marimba. You can also add your own MP3, WAV or OGG file for any event. The file is copied into the Tomatick data folder, so it keeps working if you move or delete the original. The volume slider and the mute switch apply to every alert and do not depend on the volume of the system. Each sound has a **Test** button.

## The tray icon

The tray menu shows or hides the widget, starts, pauses or resumes the timer, skips to the next phase, switches the mode, opens the statistics and the settings, and quits. When a newer version exists, a line about it appears at the top. A left click on the tray icon shows or hides the widget (Windows and macOS).

## Keyboard shortcuts

Three global shortcuts work even when another application has focus. On macOS, replace `Ctrl` with `Cmd`.

| Action | Default |
|---|---|
| Start, pause or resume | `Ctrl+Alt+P` |
| Skip to the next phase | `Ctrl+Alt+N` |
| Show or hide the widget | `Ctrl+Alt+T` |

You can change them in Settings, in the System section: click a shortcut, then press the new keys. A shortcut needs at least one modifier (Ctrl, Alt, Shift or Cmd). If a combination is already used by another application or by your system, Settings tells you. Under Wayland, global shortcuts may not work, see [Installation](./installation.md).

## Sleep and lock

Settings lets you choose what the timer does when the computer sleeps:

- **Pause the timer** (default). The timer pauses and stays paused when you come back, until you resume it. The time spent paused is kept in the history.
- **Keep the timer running.** The timer follows the clock. A phase can end while you are away, and its alerts are given when the computer wakes up.

:::caution Sleep is detected, a screen lock is not
Tomatick notices that the computer slept, but it cannot yet tell that you locked the screen. The setting is worded "sleeps or locks", and locking is planned. For now, locking the screen without sleeping does not pause the timer.
:::

## Statistics

The chart icon opens the statistics window. It only covers Pomodoro mode.

![The statistics window](/img/screenshots/statistics.png)

- **Today** shows your completed pomodoros, the focus time, and your progress toward the daily goal (8 by default).
- **Day, week and month** views show bar charts of pomodoros or focus time, with previous and next buttons.
- **Streak** counts the days in a row with at least one completed pomodoro, now and at best.
- **Completion rate** compares completed work phases with skipped and interrupted ones.
- **Best time of day** shows at which hours you complete the most pomodoros.
- **Totals** include the active days and the average focus time per active day.

A pomodoro is a completed work phase. Focus time is the time worked, without the time paused. Days follow your local time zone, and the week starts on the day your system locale says.

Everything is stored on your computer, with no limit of time. There is no export or import in this version.

## Language

Tomatick speaks English and French. By default it follows the language of your system, and you can choose another in Settings. The tray menu and the notifications change language too. To add a language, see the [translation guide](./translation.md).
