//! The tray icon and its menu (SPEC.md section 13.1).
//!
//! The menu only reports what the user picked, as an [`Action`]. The host
//! (`lib.rs`) carries the actions out, the same way it does for the global
//! shortcuts. The text is English for now and moves to the locale files with
//! the translations.

use std::sync::Mutex;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::engine::{EngineState, Mode, Snapshot};

const TRAY_ID: &str = "tomatick";

const ID_TOGGLE_WIDGET: &str = "toggle_widget";
const ID_START_PAUSE: &str = "start_pause";
const ID_SKIP: &str = "skip";
const ID_POMODORO: &str = "mode_pomodoro";
const ID_COUNTDOWN: &str = "mode_countdown";
const ID_STATS: &str = "stats";
const ID_SETTINGS: &str = "settings";
const ID_QUIT: &str = "quit";

/// What the user asked for, from the tray menu or a global shortcut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    StartPause,
    Skip,
    ToggleWidget,
    SetMode(Mode),
    OpenStats,
    OpenSettings,
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
        ID_QUIT => Action::Quit,
        _ => return None,
    })
}

/// The menu items whose text or check mark follows the state of the app.
pub struct TrayItems {
    toggle_widget: MenuItem<Wry>,
    start_pause: MenuItem<Wry>,
    skip: MenuItem<Wry>,
    pomodoro: CheckMenuItem<Wry>,
    countdown: CheckMenuItem<Wry>,
    /// What the items show now, so a refresh that changes nothing does nothing.
    shown: Mutex<Option<(Mode, EngineState, bool)>>,
}

fn start_pause_text(state: EngineState) -> &'static str {
    match state {
        EngineState::Idle => "Start",
        EngineState::Running => "Pause",
        EngineState::Paused | EngineState::AutoPaused => "Resume",
    }
}

fn widget_text(visible: bool) -> &'static str {
    if visible {
        "Hide widget"
    } else {
        "Show widget"
    }
}

/// Builds the tray icon. Fails where the system has no tray (some Linux
/// desktops), and the app then runs without one.
pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let toggle_widget =
        MenuItem::with_id(app, ID_TOGGLE_WIDGET, widget_text(true), true, None::<&str>)?;
    let start_pause = MenuItem::with_id(
        app,
        ID_START_PAUSE,
        start_pause_text(EngineState::Idle),
        true,
        None::<&str>,
    )?;
    let skip = MenuItem::with_id(app, ID_SKIP, "Skip to next phase", true, None::<&str>)?;
    let pomodoro =
        CheckMenuItem::with_id(app, ID_POMODORO, "Pomodoro mode", true, true, None::<&str>)?;
    let countdown = CheckMenuItem::with_id(
        app,
        ID_COUNTDOWN,
        "Countdown mode",
        true,
        false,
        None::<&str>,
    )?;
    let stats = MenuItem::with_id(app, ID_STATS, "Statistics", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, ID_SETTINGS, "Settings", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit Tomatick", true, None::<&str>)?;
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
        .tooltip("Tomatick")
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
        toggle_widget,
        start_pause,
        skip,
        pomodoro,
        countdown,
        shown: Mutex::new(None),
    });
    Ok(())
}

/// Brings the menu in line with the state of the timer and the widget.
/// Cheap to call often: nothing happens unless something changed.
pub fn refresh(app: &AppHandle, snapshot: &Snapshot, widget_visible: bool) {
    let Some(items) = app.try_state::<TrayItems>() else {
        return;
    };
    let key = (snapshot.mode, snapshot.state, widget_visible);
    {
        let mut shown = items
            .shown
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if *shown == Some(key) {
            return;
        }
        *shown = Some(key);
    }
    let _ = items.toggle_widget.set_text(widget_text(widget_visible));
    let _ = items.start_pause.set_text(start_pause_text(snapshot.state));
    let _ = items.skip.set_enabled(snapshot.mode == Mode::Pomodoro);
    let _ = items.pomodoro.set_checked(snapshot.mode == Mode::Pomodoro);
    let _ = items
        .countdown
        .set_checked(snapshot.mode == Mode::Countdown);
}
