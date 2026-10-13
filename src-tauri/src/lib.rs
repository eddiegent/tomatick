pub mod alerts;
pub mod engine;
pub mod overlay;
pub mod storage;

use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::ipc::Response;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewUrl,
    WebviewWindowBuilder,
};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;

use alerts::{SOUND_EXTENSIONS, SOUNDS_DIR};
use engine::{Engine, EngineState, Event, Exercise, Mode, Snapshot, SystemClock, TimerSettings};
use overlay::{OVERLAY_WINDOW, OverlayKind, overlay_url};
use storage::{AppSettings, Store, clean_exercises};

/// How often the engine is ticked.
const TICK_INTERVAL: Duration = Duration::from_millis(250);

/// Emitted after every command and while a timer runs. Payload: [`Snapshot`].
const STATE_EVENT: &str = "engine:state";
/// Emitted for each engine event (phase recorded, alert, segment change).
/// Payload: [`Event`].
const ENGINE_EVENT: &str = "engine:event";

/// Emitted after the settings were saved. Payload: [`SettingsPayload`].
const SETTINGS_EVENT: &str = "settings:changed";

/// Label of the settings window.
const SETTINGS_WINDOW: &str = "settings";

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
    }
    let _ = app.emit(STATE_EVENT, snapshot);
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
        let (title, body) = alerts::notification_text(event, label);
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

fn bring_widget_to_front(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(WIDGET_WINDOW) {
        let _ = window.show();
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
    let (events, snapshot) = {
        let mut engine = lock(engine);
        let events = action(&mut engine);
        (events, engine.snapshot())
    };
    publish(app, &events, &snapshot);
}

/// Like [`with_engine`] for commands that can fail validation. The new state
/// is published either way, and a failure is returned to the caller.
fn run_checked<E: ToString>(
    app: &AppHandle,
    engine: &SharedEngine,
    action: impl FnOnce(&mut Engine<SystemClock>) -> Result<(), E>,
) -> Result<(), String> {
    let (result, snapshot) = {
        let mut engine = lock(engine);
        let result = action(&mut engine);
        (result, engine.snapshot())
    };
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
    let picked = app
        .dialog()
        .file()
        .set_title("Choose an alert sound")
        .add_filter("Audio", SOUND_EXTENSIONS)
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
    settings: AppSettings,
    exercises: Vec<Exercise>,
) -> Result<SettingsPayload, String> {
    settings.validate().map_err(|e| e.to_string())?;

    let (previous, saved_exercises) = {
        let mut store = lock_store(&store);
        let previous = store.load_settings().map_err(|e| e.to_string())?;
        store.save_settings(&settings).map_err(|e| e.to_string())?;
        let saved = store.save_exercises(exercises).map_err(|e| e.to_string())?;
        (previous, saved)
    };

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

    let payload = SettingsPayload {
        settings,
        exercises: saved_exercises,
    };
    let _ = app.emit(SETTINGS_EVENT, payload.clone());
    Ok(payload)
}

/// Opens the settings window, or brings it to the front if it is open.
#[tauri::command]
async fn settings_open(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW) {
        let _ = window.unminimize();
        return window.set_focus().map_err(|e| e.to_string());
    }
    WebviewWindowBuilder::new(&app, SETTINGS_WINDOW, WebviewUrl::App("index.html".into()))
        .title("Tomatick settings")
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
        loop {
            thread::sleep(TICK_INTERVAL);
            let engine = app.state::<SharedEngine>();
            let (events, snapshot) = {
                let mut engine = lock(&engine);
                let events = engine.tick();
                (events, engine.snapshot())
            };
            if !events.is_empty() || snapshot.state == EngineState::Running {
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
        .setup(|app| {
            let store = open_store(app.handle());
            let engine = build_engine(&store);
            app.manage(Mutex::new(engine));
            app.manage(Mutex::new(store));
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
            overlay_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tomatick");
}
