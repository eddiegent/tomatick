//! The tray icon and its menu (SPEC.md section 13.1).
//!
//! The menu only reports what the user picked, as an [`Action`]. The host
//! (`lib.rs`) carries the actions out, the same way it does for the global
//! shortcuts. The text comes from the locale files.

use std::sync::Mutex;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::engine::{EngineState, Mode, Snapshot};
use crate::i18n::{translate, translate_with};
use crate::updater::Updater;

const TRAY_ID: &str = "tomatick";

/// Where the "update available" item goes: first in the menu, then a separator.
const UPDATE_POSITION: usize = 0;

const ID_TOGGLE_WIDGET: &str = "toggle_widget";
const ID_START_PAUSE: &str = "start_pause";
const ID_SKIP: &str = "skip";
const ID_POMODORO: &str = "mode_pomodoro";
const ID_COUNTDOWN: &str = "mode_countdown";
const ID_STATS: &str = "stats";
const ID_SETTINGS: &str = "settings";
const ID_QUIT: &str = "quit";
const ID_UPDATE: &str = "update";

/// What the user asked for, from the tray menu or a global shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    StartPause,
    Skip,
    ToggleWidget,
    SetMode(Mode),
    OpenStats,
    OpenSettings,
    /// Open the page of the newer release in the browser.
    OpenUpdatePage,
    Quit,
}

fn action_for(id: &str) -> Option<Action> {
    Some(match id {
        ID_TOGGLE_WIDGET => Action::ToggleWidget,
        ID_START_PAUSE => Action::StartPause,
        ID_SKIP => Action::Skip,
        ID_POMODORO => Action::SetMode(Mode::Pomodoro),
        ID_COUNTDOWN => Action::SetMode(Mode::Countdown),
        ID_STATS => Action::OpenStats,
        ID_SETTINGS => Action::OpenSettings,
        ID_UPDATE => Action::OpenUpdatePage,
        ID_QUIT => Action::Quit,
        _ => return None,
    })
}

/// What the menu shows now: the mode, the state of the timer, whether the
/// widget is visible, the language, and the version offered by the update
/// check, if any.
type Shown = (Mode, EngineState, bool, &'static str, Option<String>);

/// The menu items whose text or check mark follows the state of the app.
pub struct TrayItems {
    menu: Menu<Wry>,
    /// Only in the menu while a newer version is known.
    update: MenuItem<Wry>,
    update_separator: PredefinedMenuItem<Wry>,
    update_in_menu: Mutex<bool>,
    toggle_widget: MenuItem<Wry>,
    start_pause: MenuItem<Wry>,
    skip: MenuItem<Wry>,
    pomodoro: CheckMenuItem<Wry>,
    countdown: CheckMenuItem<Wry>,
    stats: MenuItem<Wry>,
    settings: MenuItem<Wry>,
    quit: MenuItem<Wry>,
    /// What the items show now, so a refresh that changes nothing does nothing.
    shown: Mutex<Option<Shown>>,
}

fn start_pause_text(language: &str, state: EngineState) -> String {
    translate(
        language,
        match state {
            EngineState::Idle => "tray.start",
            EngineState::Running => "tray.pause",
            EngineState::Paused | EngineState::AutoPaused => "tray.resume",
        },
    )
}

fn widget_text(language: &str, visible: bool) -> String {
    translate(
        language,
        if visible {
            "tray.hideWidget"
        } else {
            "tray.showWidget"
        },
    )
}

/// Builds the tray icon. Fails where the system has no tray (some Linux
/// desktops), and the app then runs without one.
pub fn build(app: &AppHandle, language: &str) -> tauri::Result<()> {
    let text = |key: &str| translate(language, key);
    let toggle_widget = MenuItem::with_id(
        app,
        ID_TOGGLE_WIDGET,
        widget_text(language, true),
        true,
        None::<&str>,
    )?;
    let start_pause = MenuItem::with_id(
        app,
        ID_START_PAUSE,
        start_pause_text(language, EngineState::Idle),
        true,
        None::<&str>,
    )?;
    let skip = MenuItem::with_id(app, ID_SKIP, text("tray.skip"), true, None::<&str>)?;
    let pomodoro = CheckMenuItem::with_id(
        app,
        ID_POMODORO,
        text("tray.pomodoroMode"),
        true,
        true,
        None::<&str>,
    )?;
    let countdown = CheckMenuItem::with_id(
        app,
        ID_COUNTDOWN,
        text("tray.countdownMode"),
        true,
        false,
        None::<&str>,
    )?;
    let stats = MenuItem::with_id(app, ID_STATS, text("tray.stats"), true, None::<&str>)?;
    let settings = MenuItem::with_id(app, ID_SETTINGS, text("tray.settings"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ID_QUIT, text("tray.quit"), true, None::<&str>)?;
    let update = MenuItem::with_id(app, ID_UPDATE, text("tray.update"), true, None::<&str>)?;
    let update_separator = PredefinedMenuItem::separator(app)?;
    let first = PredefinedMenuItem::separator(app)?;
    let second = PredefinedMenuItem::separator(app)?;
    let third = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &toggle_widget,
            &start_pause,
            &skip,
            &first,
            &pomodoro,
            &countdown,
            &second,
            &stats,
            &settings,
            &third,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip(text("tray.tooltip"))
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            if let Some(action) = action_for(event.id.as_ref()) {
                crate::run_action(app, action);
            }
        })
        .on_tray_icon_event(|tray, event| {
            // A left click shows or hides the widget. Not sent on Linux,
            // where the menu is the way in.
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                crate::run_action(tray.app_handle(), Action::ToggleWidget);
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;

    app.manage(TrayItems {
        menu,
        update,
        update_separator,
        update_in_menu: Mutex::new(false),
        toggle_widget,
        start_pause,
        skip,
        pomodoro,
        countdown,
        stats,
        settings,
        quit,
        shown: Mutex::new(None),
    });
    Ok(())
}

/// Puts the "update available" item at the top of the menu while a newer
/// version is known, and takes it out again when there is none.
fn show_update(items: &TrayItems, language: &str, version: Option<&str>) {
    let mut in_menu = items
        .update_in_menu
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    match version {
        Some(version) => {
            let _ = items.update.set_text(translate_with(
                language,
                "tray.update",
                &[("version", version)],
            ));
            if !*in_menu {
                let added = items.menu.insert(&items.update, UPDATE_POSITION).is_ok()
                    && items
                        .menu
                        .insert(&items.update_separator, UPDATE_POSITION + 1)
                        .is_ok();
                *in_menu = added;
            }
        }
        None => {
            if *in_menu {
                let _ = items.menu.remove(&items.update);
                let _ = items.menu.remove(&items.update_separator);
                *in_menu = false;
            }
        }
    }
}

/// Brings the menu in line with the state of the timer and the widget.
/// Cheap to call often: nothing happens unless something changed.
pub fn refresh(app: &AppHandle, snapshot: &Snapshot, widget_visible: bool, language: &'static str) {
    let Some(items) = app.try_state::<TrayItems>() else {
        return;
    };
    let update_version = app
        .try_state::<Updater>()
        .and_then(|updater| updater.status().available)
        .map(|available| available.version);
    let key = (
        snapshot.mode,
        snapshot.state,
        widget_visible,
        language,
        update_version.clone(),
    );
    {
        let mut shown = items
            .shown
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if shown.as_ref() == Some(&key) {
            return;
        }
        *shown = Some(key);
    }
    show_update(&items, language, update_version.as_deref());
    let _ = items
        .toggle_widget
        .set_text(widget_text(language, widget_visible));
    let _ = items
        .start_pause
        .set_text(start_pause_text(language, snapshot.state));
    let _ = items.skip.set_text(translate(language, "tray.skip"));
    let _ = items
        .pomodoro
        .set_text(translate(language, "tray.pomodoroMode"));
    let _ = items
        .countdown
        .set_text(translate(language, "tray.countdownMode"));
    let _ = items.stats.set_text(translate(language, "tray.stats"));
    let _ = items
        .settings
        .set_text(translate(language, "tray.settings"));
    let _ = items.quit.set_text(translate(language, "tray.quit"));
    let _ = items.skip.set_enabled(snapshot.mode == Mode::Pomodoro);
    let _ = items.pomodoro.set_checked(snapshot.mode == Mode::Pomodoro);
    let _ = items
        .countdown
        .set_checked(snapshot.mode == Mode::Countdown);
}
