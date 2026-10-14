---
title: Installation
sidebar_position: 1
---

# Installation

Installers for Windows, macOS and Linux are published on the [releases page](https://github.com/eddiegent/tomatick/releases). Pick the file for your system:

| System | File |
|---|---|
| Windows 10 and 11 | `.exe` installer |
| macOS (Intel and Apple silicon) | `.dmg` |
| Linux | `.AppImage`, or `.deb` for Debian and Ubuntu |

:::info Builds are not signed
Tomatick is a free project and the installers are not signed with a paid code-signing certificate. Windows and macOS therefore show a warning the first time you open it. The steps below explain how to continue. You can also build the app yourself from the source, see [Contributing](./contributing.md).
:::

## Windows

1. Download the `.exe` installer and run it.
2. If Windows SmartScreen says "Windows protected your PC", click **More info**, then **Run anyway**.
3. Windows 11 already includes the WebView2 runtime that Tomatick needs. On Windows 10 the installer downloads it if it is missing.

## macOS

1. Open the `.dmg` and drag Tomatick to the Applications folder.
2. The first time, macOS may say that it cannot verify the app. Open the **Applications** folder, hold **Control** while you click Tomatick, choose **Open**, then confirm.
3. If that is not offered, open **System Settings**, then **Privacy and Security**, scroll down and click **Open Anyway** next to the Tomatick message.

## Linux

**AppImage:** make the file executable and run it.

```bash
chmod +x Tomatick_*.AppImage
./Tomatick_*.AppImage
```

**Debian and Ubuntu:** install the `.deb` file. The package depends on WebKitGTK, and the package manager installs it for you.

```bash
sudo apt install ./Tomatick_*.deb
```

### The tray icon on Linux

The tray icon is the way back to the app when the widget is hidden. Some desktops, GNOME in particular, do not show tray icons unless an extension such as AppIndicator support is installed. Without a tray, the widget and its buttons still work.

### X11 and Wayland

Tomatick works best on X11. Under Wayland, the compositor decides what an application may do, and some features are limited:

- **Always on top:** the widget may not stay above other windows.
- **Window position:** the position is chosen by the compositor, so the widget may not come back where you left it.
- **Global shortcuts:** shortcuts pressed while another application has focus may not be delivered. The tray menu and the widget buttons still work.

Settings shows an error next to a shortcut that could not be registered, instead of failing silently. If you need all three features, log in to an X11 session.

## Updates

Tomatick asks GitHub once a day, and when it starts, whether a newer release exists. If one does, a notice appears in the tray menu and in Settings, with a link to the releases page. It never downloads or installs anything by itself. This is the only network request the app makes, and you can turn it off in Settings, in the System section.

## Where your data is stored

Settings and history are kept in one SQLite file, `tomatick.db`, in the application data folder of your system. Custom alert sounds are copied to a `sounds` folder next to it. Removing the app does not remove this folder, so delete it yourself if you want a clean start.

| System | Location |
|---|---|
| Windows | `%APPDATA%\io.github.eddiegent.tomatick` |
| macOS | `~/Library/Application Support/io.github.eddiegent.tomatick` |
| Linux | `~/.local/share/io.github.eddiegent.tomatick` |
