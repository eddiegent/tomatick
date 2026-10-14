---
title: Frequently asked questions
sidebar_position: 4
---

# Frequently asked questions

## Does Tomatick have a task list?

No, and it will not. It is a pure timer: no tasks, no accounts, no cloud.

## Does it send my data anywhere?

No. History and settings stay in a file on your computer. The only network request is the check for new versions, which asks GitHub for the latest release. You can turn it off in Settings.

## Can I export my history?

Not in this version. The history is a SQLite file, `tomatick.db`, in the application data folder (see [Installation](./installation.md#where-your-data-is-stored)), so you can read it with any SQLite tool.

## Windows or macOS warns me when I open it

The installers are not signed with a paid certificate, so both systems warn about an unknown publisher. See [Installation](./installation.md) for how to continue.

## The widget does not stay on top, or my shortcuts do not work (Linux)

You are probably running Wayland. Always-on-top, window position and global shortcuts depend on the compositor there, and they work on X11. See [Installation](./installation.md#x11-and-wayland).

## There is no tray icon

Some Linux desktops, GNOME in particular, hide tray icons unless an extension adds support for them. The widget and its buttons keep working. On other systems, check the hidden icons area of the taskbar.

## I locked my screen and the timer kept running

Tomatick detects that the computer went to sleep, but not yet that the screen was locked. Use the pause button, or let the computer sleep.

## Why does a skipped work phase not count?

Only a work phase that ran to its end is a pomodoro. A skipped one is recorded as skipped, but it still moves you forward in the cycle, so you reach your long break.

## Can I run a countdown and a Pomodoro at the same time?

No. Only one mode is active at a time, and switching mode resets the timer you leave.

## What happens if I quit during a phase?

The phase is recorded as interrupted the next time you start the app. A half-finished phase is not restored.

## How do I start over with the default cycle?

In Settings, press **Reset to classic**, then **Save**.

## How do I remove Tomatick and its data?

Uninstall it like any other application, then delete the data folder listed in [Installation](./installation.md#where-your-data-is-stored).

## I found a bug or I have an idea

Please [open an issue](https://github.com/eddiegent/tomatick/issues). If you want to fix it yourself, see [Contributing](./contributing.md).
