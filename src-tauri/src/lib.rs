pub mod alerts;
pub mod engine;
pub mod history;
pub mod i18n;
pub mod overlay;
pub mod placement;
pub mod power;
pub mod shortcuts;
pub mod storage;
pub mod tray;

use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::ipc::Response;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewUrl,
    WebviewWindowBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;

use alerts::{SOUND_EXTENSIONS, SOUNDS_DIR};
use engine::{Engine, EngineState, Event, Exercise, Mode, Snapshot, SystemClock, TimerSettings};
use history::{LOCAL_TIME, OpenPhase, Period, Stats, StatsQuery};
use overlay::{OVERLAY_WINDOW, OverlayKind, overlay_url};
use power::{GapDetector, PowerEvent, PowerListener};
use shortcuts::{ShortcutAction, ShortcutSettings};
use storage::{AppSettings, DEFAULT_DAILY_GOAL, Store, clean_exercises};
use tray::Action;

/// The widget is checked against the screens every this many ticks (5 s).
const SCREEN_CHECK_TICKS: u32 = 20;

/// How often the engine is ticked.
const TICK_INTERVAL: Duration = Duration::from_millis(250);

/// Emitted after every command and while a timer runs. Payload: [`Snapshot`].
const STATE_EVENT: &str = "engine:state";
/// Emitted for each engine event (phase recorded, alert, segment change).
/// Payload: [`Event`].
const ENGINE_EVENT: &str = "engine:event";

/// Emitted after the settings were saved. Payload: [`SettingsPayload`].
const SETTINGS_EVENT: &str = "settings:changed";

/// Emitted after a phase was added to the history, for an open statistics
/// window to refresh itself. No payload.
const HISTORY_EVENT: &str = "history:changed";

/// Label of the settings window.
const SETTINGS_WINDOW: &str = "settings";

/// Label of the statistics window.
const STATS_WINDOW: &str = "stats";

/// Label of the widget window.
const WIDGET_WINDOW: &str = "main";

/// Sent to an overlay that is already open, to show another kind.
/// Payload: [`OverlayRequest`].
const OVERLAY_EVENT: &str = "overlay:open";

/// What the overlay page is asked to show.
#[derive(Clone, Copy, Debug, Serialize)]
struct OverlayRequest {
    kind: OverlayKind,
    preview: bool,
}

type SharedEngine = Mutex<Engine<SystemClock>>;
type SharedStore = Mutex<Store>;

fn lock_store(store: &SharedStore) -> MutexGuard<'_, Store> {
    store
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Everything the settings dialog edits.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SettingsPayload {
    settings: AppSettings,
    exercises: Vec<Exercise>,
}

fn lock(engine: &SharedEngine) -> MutexGuard<'_, Engine<SystemClock>> {
    // A panic while holding the lock must not take the whole app down.
    engine
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn publish(app: &AppHandle, events: &[Event], snapshot: &Snapshot) {
    for event in events {
        let _ = app.emit(ENGINE_EVENT, event);
        if let Event::Alert { event, label } = event {
            run_alert(app, *event, label.as_deref());
        }
        record_history(app, event, snapshot);
    }
    let _ = app.emit(STATE_EVENT, snapshot);
    tray::refresh(app, snapshot, widget_visible(app), current_language(app));
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_millis() as u64)
}

/// Keeps the history: a finished phase is stored, and a started one is noted
/// so that a crash can still be recorded at the next launch.
fn record_history(app: &AppHandle, event: &Event, snapshot: &Snapshot) {
    let recorded = {
        let store = app.state::<SharedStore>();
        let store = lock_store(&store);
        match event {
            Event::PhaseRecorded(record) => {
                if let Err(error) = store.record_phase(record) {
                    eprintln!("Tomatick: cannot record the phase: {error}");
                }
                let _ = store.clear_open_phase();
                true
            }
            Event::PhaseStarted { kind } => {
                let phase = OpenPhase {
                    kind: *kind,
                    started_unix_ms: unix_now_ms(),
                    planned_secs: u32::try_from(snapshot.total_ms / 1000).unwrap_or(u32::MAX),
                };
                if let Err(error) = store.set_open_phase(&phase) {
                    eprintln!("Tomatick: cannot note the running phase: {error}");
                }
                false
            }
            _ => false,
        }
    };
    if recorded {
        let _ = app.emit(HISTORY_EVENT, ());
    }
}

/// Called when the app quits: a phase in progress is recorded as interrupted.
fn finish_session(app: &AppHandle) {
    let events = lock(&app.state::<SharedEngine>()).shutdown();
    let store = app.state::<SharedStore>();
    let store = lock_store(&store);
    for event in events {
        if let Event::PhaseRecorded(record) = event
            && let Err(error) = store.record_phase(&record)
        {
            eprintln!("Tomatick: cannot record the phase: {error}");
        }
    }
    let _ = store.clear_open_phase();
}

/// Does the parts of an alert that belong to the system: the notification and
/// raising the widget. The sound is played by the widget window, which hears
/// the same engine event.
fn run_alert(app: &AppHandle, event: engine::AlertEvent, label: Option<&str>) {
    let alerts = {
        let store = app.state::<SharedStore>();
        let settings = lock_store(&store).load_settings().unwrap_or_default();
        settings.alerts.for_event(event).clone()
    };

    if alerts.notification {
        let (title, body) = i18n::notification_text(current_language(app), event, label);
        if let Err(error) = app.notification().builder().title(title).body(body).show() {
            eprintln!("Tomatick: cannot show the notification: {error}");
        }
    }
    if alerts.bring_to_front {
        bring_widget_to_front(app);
    }
    if alerts.overlay
        && let Some(kind) = OverlayKind::for_event(event)
    {
        open_overlay_later(app, kind, false);
    }
}

/// Whether the widget window is on screen.
fn widget_visible(app: &AppHandle) -> bool {
    app.get_webview_window(WIDGET_WINDOW)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(true)
}

fn toggle_widget(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WIDGET_WINDOW) {
        if window.is_visible().unwrap_or(true) {
            let _ = window.hide();
        } else {
            bring_widget_to_front(app);
        }
    }
    let snapshot = lock(&app.state::<SharedEngine>()).snapshot();
    tray::refresh(app, &snapshot, widget_visible(app), current_language(app));
}

/// Carries out something asked for from the tray menu or a global shortcut.
pub(crate) fn run_action(app: &AppHandle, action: Action) {
    let engine = app.state::<SharedEngine>();
    match action {
        Action::StartPause => with_engine(app, &engine, |e| e.toggle()),
        Action::Skip => with_engine(app, &engine, |e| e.skip()),
        Action::SetMode(mode) => with_engine(app, &engine, |e| e.set_mode(mode)),
        Action::ToggleWidget => toggle_widget(app),
        // Windows are built from a thread of their own: building one from
        // the thread that handles a menu click can deadlock on Windows.
        Action::OpenStats => {
            let app = app.clone();
            thread::spawn(move || {
                if let Err(error) = open_stats_window(&app) {
                    eprintln!("Tomatick: cannot open the statistics: {error}");
                }
            });
        }
        Action::OpenSettings => {
            let app = app.clone();
            thread::spawn(move || {
                if let Err(error) = open_settings_window(&app) {
                    eprintln!("Tomatick: cannot open the settings: {error}");
                }
            });
        }
        Action::Quit => app.exit(0),
    }
}

fn action_for_shortcut(action: ShortcutAction) -> Action {
    match action {
        ShortcutAction::StartPause => Action::StartPause,
        ShortcutAction::Skip => Action::Skip,
        ShortcutAction::ToggleWidget => Action::ToggleWidget,
    }
}

/// How one global shortcut ended up: registered, or why not.
#[derive(Clone, Debug, Serialize)]
struct ShortcutStatus {
    action: ShortcutAction,
    accelerator: String,
    /// `None` when registered, or when the shortcut is not bound.
    error: Option<String>,
}

/// What the settings dialog shows about the desktop integration.
#[derive(Clone, Debug, Default, Serialize)]
struct DesktopStatus {
    shortcuts: Vec<ShortcutStatus>,
    /// Why launch at login could not be switched, if it could not.
    autostart_error: Option<String>,
}

/// Which registered shortcut does what, and how the last registration went.
#[derive(Default)]
struct Desktop {
    /// The language of the text the Rust side produces.
    language: Mutex<&'static str>,
    bindings: Mutex<Vec<(u32, Action)>>,
    status: Mutex<DesktopStatus>,
}

/// The language for the tray, the notifications and the window titles.
fn current_language(app: &AppHandle) -> &'static str {
    match app.try_state::<Desktop>() {
        Some(desktop) => {
            let language = *lock_desktop(&desktop.language);
            if language.is_empty() {
                i18n::FALLBACK
            } else {
                language
            }
        }
        None => i18n::FALLBACK,
    }
}

/// Picks the language from the setting, "system" meaning the language of
/// the operating system, and remembers it.
fn apply_language(app: &AppHandle, setting: &str) {
    let system = sys_locale::get_locale();
    let language = i18n::resolve(setting, system.as_deref());
    *lock_desktop(&app.state::<Desktop>().language) = language;
}

fn lock_desktop<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Registers the global shortcuts, replacing the previous ones. A shortcut
/// that another application already holds, or that the system refuses (for
/// example under Wayland), is reported in the status and does not stop the
/// others.
fn apply_shortcuts(app: &AppHandle, settings: &ShortcutSettings) {
    let manager = app.global_shortcut();
    let _ = manager.unregister_all();

    let mut bindings = Vec::new();
    let mut statuses = Vec::new();
    for (action, text) in settings.bindings() {
        let mut error = None;
        if !text.is_empty() {
            let registered = text
                .parse::<Shortcut>()
                .map_err(|e| e.to_string())
                .and_then(|shortcut| {
                    manager
                        .register(shortcut)
                        .map(|()| shortcut)
                        .map_err(|e| e.to_string())
                });
            match registered {
                Ok(shortcut) => bindings.push((shortcut.id(), action_for_shortcut(action))),
                Err(message) => {
                    eprintln!("Tomatick: cannot register {text}: {message}");
                    error = Some(message);
                }
            }
        }
        statuses.push(ShortcutStatus {
            action,
            accelerator: text.to_string(),
            error,
        });
    }

    let desktop = app.state::<Desktop>();
    *lock_desktop(&desktop.bindings) = bindings;
    lock_desktop(&desktop.status).shortcuts = statuses;
}

/// Switches launch at login to match the setting. Nothing is touched when it
/// already matches, so the default (off) leaves the system alone.
fn apply_autostart(app: &AppHandle, wanted: bool) {
    let manager = app.autolaunch();
    let error = match manager.is_enabled() {
        Ok(current) if current == wanted => None,
        _ => {
            let result = if wanted {
                manager.enable()
            } else {
                manager.disable()
            };
            result.err().map(|e| e.to_string())
        }
    };
    if let Some(message) = &error {
        eprintln!("Tomatick: cannot change launch at login: {message}");
    }
    lock_desktop(&app.state::<Desktop>().status).autostart_error = error;
}

#[tauri::command]
fn desktop_status(desktop: State<'_, Desktop>) -> DesktopStatus {
    lock_desktop(&desktop.status).clone()
}

/// Moves the widget onto a screen if it is not on one, for example after a
/// monitor was unplugged. Does nothing while it is hidden or already visible
/// on a screen.
fn keep_widget_on_screen(app: &AppHandle) {
    let Some(window) = app.get_webview_window(WIDGET_WINDOW) else {
        return;
    };
    if !window.is_visible().unwrap_or(false) {
        return;
    }
    let (Ok(position), Ok(size), Ok(monitors)) = (
        window.outer_position(),
        window.outer_size(),
        window.available_monitors(),
    ) else {
        return;
    };
    let screens: Vec<placement::Rect> = monitors
        .iter()
        .map(|monitor| {
            let area = monitor.work_area();
            placement::Rect::new(
                area.position.x,
                area.position.y,
                area.size.width,
                area.size.height,
            )
        })
        .collect();
    let widget = placement::Rect::new(position.x, position.y, size.width, size.height);
    if let Some((x, y)) = placement::reposition(widget, &screens) {
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }
}

fn bring_widget_to_front(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WIDGET_WINDOW) {
        let _ = window.show();
        keep_widget_on_screen(app);
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

/// Opens the overlay from a thread of its own. Building a window from the
/// thread that handles a command can deadlock on Windows.
fn open_overlay_later(app: &AppHandle, kind: OverlayKind, preview: bool) {
    let app = app.clone();
    thread::spawn(move || {
        if let Err(error) = open_overlay(&app, kind, preview) {
            eprintln!("Tomatick: cannot open the overlay: {error}");
        }
    });
}

/// Opens the full-screen overlay on the display that holds the widget, or
/// brings the open one forward and tells it what to show.
fn open_overlay(app: &AppHandle, kind: OverlayKind, preview: bool) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(OVERLAY_WINDOW) {
        let _ = app.emit_to(
            OVERLAY_WINDOW,
            OVERLAY_EVENT,
            OverlayRequest { kind, preview },
        );
        let _ = window.show();
        return window.set_focus().map_err(|e| e.to_string());
    }

    // The monitor of the widget, else the primary one, else a default size.
    let monitor = app
        .get_webview_window(WIDGET_WINDOW)
        .and_then(|widget| widget.current_monitor().ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());

    // Built hidden and placed in physical pixels, so a monitor with another
    // scale factor than the primary one still gets covered completely.
    let window = WebviewWindowBuilder::new(
        app,
        OVERLAY_WINDOW,
        WebviewUrl::App(overlay_url(kind, preview).into()),
    )
    .title("Tomatick")
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .resizable(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible(false)
    .focused(true)
    .build()
    .map_err(|e| e.to_string())?;

    match monitor {
        Some(monitor) => {
            let position = monitor.position();
            let size = monitor.size();
            let _ = window.set_position(PhysicalPosition::new(position.x, position.y));
            let _ = window.set_size(PhysicalSize::new(size.width, size.height));
        }
        None => {
            let _ = window.maximize();
        }
    }
    window.show().map_err(|e| e.to_string())?;
    let _ = window.set_focus();
    Ok(())
}

/// Opens the overlay with sample content, for the preview in the settings.
/// The timer is not touched.
#[tauri::command]
async fn overlay_preview(app: AppHandle, kind: OverlayKind) -> Result<(), String> {
    open_overlay(&app, kind, true)
}

/// Runs a command on the engine, then publishes its events and the new state.
fn with_engine(
    app: &AppHandle,
    engine: &SharedEngine,
    action: impl FnOnce(&mut Engine<SystemClock>) -> Vec<Event>,
) {
    let (events, snapshot, rotation) = {
        let mut engine = lock(engine);
        let events = action(&mut engine);
        (events, engine.snapshot(), engine.rotation_position())
    };
    remember_rotation(app, rotation);
    publish(app, &events, &snapshot);
}

/// Saves where the exercise rotation stands, so it carries on after a
/// restart. Nothing is written when the position has not changed.
fn remember_rotation(app: &AppHandle, position: usize) {
    let store = app.state::<SharedStore>();
    if let Err(error) = lock_store(&store).save_rotation(position) {
        eprintln!("Tomatick: cannot save the exercise rotation: {error}");
    }
}

/// Like [`with_engine`] for commands that can fail validation. The new state
/// is published either way, and a failure is returned to the caller.
fn run_checked<E: ToString>(
    app: &AppHandle,
    engine: &SharedEngine,
    action: impl FnOnce(&mut Engine<SystemClock>) -> Result<(), E>,
) -> Result<(), String> {
    let (result, snapshot, rotation) = {
        let mut engine = lock(engine);
        let result = action(&mut engine);
        (result, engine.snapshot(), engine.rotation_position())
    };
    remember_rotation(app, rotation);
    publish(app, &[], &snapshot);
    result.map_err(|error| error.to_string())
}

#[tauri::command]
fn engine_snapshot(engine: State<'_, SharedEngine>) -> Snapshot {
    lock(&engine).snapshot()
}

#[tauri::command]
fn engine_start(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| e.start());
}

#[tauri::command]
fn engine_pause(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| {
        e.pause();
        Vec::new()
    });
}

#[tauri::command]
fn engine_resume(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| {
        e.resume();
        Vec::new()
    });
}

#[tauri::command]
fn engine_toggle(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| e.toggle());
}

#[tauri::command]
fn engine_skip(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| e.skip());
}

#[tauri::command]
fn engine_skip_and_start(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| e.skip_and_start());
}

#[tauri::command]
fn engine_reset(app: AppHandle, engine: State<'_, SharedEngine>) {
    with_engine(&app, &engine, |e| e.reset());
}

#[tauri::command]
fn engine_extend(app: AppHandle, engine: State<'_, SharedEngine>, seconds: u32) {
    with_engine(&app, &engine, |e| {
        e.extend(seconds);
        Vec::new()
    });
}

#[tauri::command]
fn engine_set_mode(app: AppHandle, engine: State<'_, SharedEngine>, mode: Mode) {
    with_engine(&app, &engine, |e| e.set_mode(mode));
}

#[tauri::command]
fn engine_set_countdown(
    app: AppHandle,
    engine: State<'_, SharedEngine>,
    store: State<'_, SharedStore>,
    seconds: u32,
) -> Result<(), String> {
    run_checked(&app, &engine, |e| e.set_countdown(seconds))?;
    // Remember the value for the next launch and tell the settings dialog.
    let store = lock_store(&store);
    let mut settings = store.load_settings().map_err(|e| e.to_string())?;
    settings.countdown_secs = seconds;
    store.save_settings(&settings).map_err(|e| e.to_string())?;
    let exercises = store.load_exercises().map_err(|e| e.to_string())?;
    let _ = app.emit(
        SETTINGS_EVENT,
        SettingsPayload {
            settings,
            exercises,
        },
    );
    Ok(())
}

#[tauri::command]
fn engine_update_settings(
    app: AppHandle,
    engine: State<'_, SharedEngine>,
    settings: TimerSettings,
) -> Result<(), String> {
    run_checked(&app, &engine, |e| e.update_settings(settings))
}

#[tauri::command]
fn engine_set_exercises(app: AppHandle, engine: State<'_, SharedEngine>, exercises: Vec<Exercise>) {
    with_engine(&app, &engine, |e| {
        e.set_exercises(exercises);
        Vec::new()
    });
}

#[tauri::command]
fn settings_get(store: State<'_, SharedStore>) -> Result<SettingsPayload, String> {
    let store = lock_store(&store);
    Ok(SettingsPayload {
        settings: store.load_settings().map_err(|e| e.to_string())?,
        exercises: store.load_exercises().map_err(|e| e.to_string())?,
    })
}

/// The classic Pomodoro profile, for the "reset to classic" button. The
/// dialog fills its form with it and the user saves when happy.
#[tauri::command]
fn settings_classic() -> TimerSettings {
    TimerSettings::classic()
}

/// The folder that holds the custom alert sounds.
fn sounds_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_data_dir()
        .map(|dir| dir.join(SOUNDS_DIR))
        .map_err(|e| e.to_string())
}

/// Lets the user pick an audio file and copies it into the app data folder.
/// Returns the stored file name, or `None` if the dialog was cancelled.
#[tauri::command]
async fn sound_add(app: AppHandle) -> Result<Option<String>, String> {
    let language = current_language(&app);
    let picked = app
        .dialog()
        .file()
        .set_title(i18n::translate(language, "dialog.chooseSound"))
        .add_filter(i18n::translate(language, "dialog.audio"), SOUND_EXTENSIONS)
        .blocking_pick_file();
    let Some(picked) = picked else {
        return Ok(None);
    };
    let source = picked.into_path().map_err(|e| e.to_string())?;
    alerts::import_sound(&source, &sounds_dir(&app)?)
        .map(Some)
        .map_err(|e| e.to_string())
}

/// The bytes of a custom sound, for the player in the webview.
#[tauri::command]
fn sound_data(app: AppHandle, file: String) -> Result<Response, String> {
    let path = alerts::sound_path(&sounds_dir(&app)?, &file).map_err(|e| e.to_string())?;
    std::fs::read(path)
        .map(Response::new)
        .map_err(|e| e.to_string())
}

/// Validates, stores and applies the settings and the exercise list.
#[tauri::command]
fn settings_save(
    app: AppHandle,
    engine: State<'_, SharedEngine>,
    store: State<'_, SharedStore>,
    mut settings: AppSettings,
    exercises: Vec<Exercise>,
) -> Result<SettingsPayload, String> {
    settings.validate().map_err(|e| e.to_string())?;
    settings.shortcuts = settings.shortcuts.normalized().map_err(|e| e.to_string())?;

    let (previous, saved_exercises) = {
        let mut store = lock_store(&store);
        let previous = store.load_settings().map_err(|e| e.to_string())?;
        store.save_settings(&settings).map_err(|e| e.to_string())?;
        let saved = store.save_exercises(exercises).map_err(|e| e.to_string())?;
        (previous, saved)
    };

    // Before the engine publishes, so the tray is relabeled by that publish.
    apply_language(&app, &settings.language);
    let countdown_changed = previous.countdown_secs != settings.countdown_secs;
    run_checked(&app, &engine, |e| {
        e.update_settings(settings.timer.clone())?;
        e.set_exercises(saved_exercises.clone());
        // Changing the countdown resets it, so leave a running one alone.
        let snapshot = e.snapshot();
        let busy = snapshot.mode == Mode::Countdown && snapshot.state != EngineState::Idle;
        if countdown_changed && !busy {
            e.set_countdown(settings.countdown_secs)?;
        }
        Ok::<(), engine::EngineError>(())
    })?;

    apply_shortcuts(&app, &settings.shortcuts);
    apply_autostart(&app, settings.launch_at_login);

    let payload = SettingsPayload {
        settings,
        exercises: saved_exercises,
    };
    let _ = app.emit(SETTINGS_EVENT, payload.clone());
    Ok(payload)
}

/// Opens the settings window, or brings it to the front if it is open.
/// The statistics of a day, a week or a month, in the local time.
#[tauri::command]
fn stats_get(
    store: State<'_, SharedStore>,
    period: Period,
    offset: i32,
    week_start: u8,
) -> Result<Stats, String> {
    let store = lock_store(&store);
    let goal = store
        .load_settings()
        .map_or(DEFAULT_DAILY_GOAL, |settings| settings.daily_goal);
    let today = store.local_today().map_err(|e| e.to_string())?;
    let query = StatsQuery {
        period,
        offset,
        week_start,
    };
    store
        .stats(&query, goal, &today, LOCAL_TIME)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn stats_open(app: AppHandle) -> Result<(), String> {
    open_stats_window(&app)
}

fn open_stats_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(STATS_WINDOW) {
        let _ = window.unminimize();
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(app, STATS_WINDOW, WebviewUrl::App("index.html".into()))
        .title(i18n::translate(current_language(app), "windowTitle.stats"))
        .inner_size(600.0, 760.0)
        .min_inner_size(460.0, 520.0)
        .resizable(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn settings_open(app: AppHandle) -> Result<(), String> {
    open_settings_window(&app)
}

fn open_settings_window(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
        let _ = window.unminimize();
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(app, SETTINGS_WINDOW, WebviewUrl::App("index.html".into()))
        .title(i18n::translate(
            current_language(app),
            "windowTitle.settings",
        ))
        .inner_size(540.0, 700.0)
        .min_inner_size(440.0, 480.0)
        .resizable(true)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Ticks the engine on a background thread and publishes what happens.
fn spawn_ticker(app: AppHandle) {
    thread::spawn(move || {
        let mut power: Box<dyn PowerListener> = Box::new(GapDetector::default());
        let mut ticks: u32 = 0;
        loop {
            thread::sleep(TICK_INTERVAL);
            // Every few seconds, in case a screen was unplugged.
            ticks = ticks.wrapping_add(1);
            if ticks % SCREEN_CHECK_TICKS == 0 {
                keep_widget_on_screen(&app);
            }
            let engine = app.state::<SharedEngine>();
            let (events, snapshot, rotation, suspended) = {
                let mut engine = lock(&engine);
                // A sleep is noticed on wake, before the tick, so a phase
                // that "ended" during the sleep is paused instead.
                let mut suspended = false;
                for event in power.poll(engine.now_ms()) {
                    let PowerEvent::Suspended { since_ms } = event;
                    engine.on_system_suspend_at(since_ms);
                    suspended = true;
                }
                let events = engine.tick();
                (
                    events,
                    engine.snapshot(),
                    engine.rotation_position(),
                    suspended,
                )
            };
            // The rotation only moves when something happens, so the
            // database is left alone on a quiet tick.
            if !events.is_empty() {
                remember_rotation(&app, rotation);
            }
            if suspended || !events.is_empty() || snapshot.state == EngineState::Running {
                publish(&app, &events, &snapshot);
            }
        }
    });
}

/// Opens the database in the app data folder. If that fails, the app still
/// runs with settings kept in memory only.
fn open_store(app: &AppHandle) -> Store {
    let on_disk = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())
        .and_then(|dir| {
            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
            Store::open(&dir.join("tomatick.db")).map_err(|e| e.to_string())
        });
    match on_disk {
        Ok(store) => store,
        Err(error) => {
            eprintln!("Tomatick: cannot open the database, settings will not be saved: {error}");
            Store::open_in_memory().expect("an in-memory database always opens")
        }
    }
}

fn build_engine(store: &Store) -> Engine<SystemClock> {
    let settings = store.load_settings().unwrap_or_default();
    let exercises = clean_exercises(store.load_exercises().unwrap_or_default());
    let mut engine = Engine::new(SystemClock::new(), settings.timer, exercises)
        .or_else(|_| Engine::new(SystemClock::new(), TimerSettings::classic(), Vec::new()))
        .expect("the classic settings are valid");
    let _ = engine.set_countdown(settings.countdown_secs);
    engine.restore_rotation(store.load_rotation().unwrap_or(0));
    engine
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        // Remember where the widget was left, on which monitor. Only the
        // position: the size is set by the widget size setting.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let action = {
                        let desktop = app.state::<Desktop>();
                        let bindings = lock_desktop(&desktop.bindings);
                        bindings
                            .iter()
                            .find(|(id, _)| *id == shortcut.id())
                            .map(|(_, action)| *action)
                    };
                    if let Some(action) = action {
                        run_action(app, action);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let store = open_store(app.handle());
            // A phase that was running when the app last stopped without
            // quitting is recorded as interrupted.
            if let Err(error) = store.recover_open_phase() {
                eprintln!("Tomatick: cannot recover the last phase: {error}");
            }
            let engine = build_engine(&store);
            let settings = store.load_settings().unwrap_or_default();
            app.manage(Mutex::new(engine));
            app.manage(Mutex::new(store));
            app.manage(Desktop::default());
            apply_language(app.handle(), &settings.language);
            apply_shortcuts(app.handle(), &settings.shortcuts);
            if let Err(error) = tray::build(app.handle(), current_language(app.handle())) {
                eprintln!("Tomatick: no tray icon: {error}");
            }
            // Some systems ask the user once; elsewhere this is always granted.
            let _ = app.notification().request_permission();
            spawn_ticker(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            engine_snapshot,
            engine_start,
            engine_pause,
            engine_resume,
            engine_toggle,
            engine_skip,
            engine_skip_and_start,
            engine_reset,
            engine_extend,
            engine_set_mode,
            engine_set_countdown,
            engine_update_settings,
            engine_set_exercises,
            settings_get,
            settings_classic,
            settings_save,
            sound_add,
            sound_data,
            settings_open,
            stats_get,
            stats_open,
            desktop_status,
            overlay_preview,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tomatick")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                finish_session(app);
            }
        });
}
