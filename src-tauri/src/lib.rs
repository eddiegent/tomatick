pub mod engine;

use std::sync::{Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, State};

use engine::{Engine, EngineState, Event, Exercise, Mode, Snapshot, SystemClock, TimerSettings};

/// How often the engine is ticked.
const TICK_INTERVAL: Duration = Duration::from_millis(250);

/// Emitted after every command and while a timer runs. Payload: [`Snapshot`].
const STATE_EVENT: &str = "engine:state";
/// Emitted for each engine event (phase recorded, alert, segment change).
/// Payload: [`Event`].
const ENGINE_EVENT: &str = "engine:event";

type SharedEngine = Mutex<Engine<SystemClock>>;

fn lock(engine: &SharedEngine) -> MutexGuard<'_, Engine<SystemClock>> {
    // A panic while holding the lock must not take the whole app down.
    engine
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn publish(app: &AppHandle, events: &[Event], snapshot: &Snapshot) {
    for event in events {
        let _ = app.emit(ENGINE_EVENT, event);
    }
    let _ = app.emit(STATE_EVENT, snapshot);
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
    seconds: u32,
) -> Result<(), String> {
    run_checked(&app, &engine, |e| e.set_countdown(seconds))
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = Engine::new(SystemClock::new(), TimerSettings::classic(), Vec::new())
        .expect("the classic settings are valid");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Remember where the widget was left, on which monitor. Only the
        // position: the size is set by the widget size setting.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(tauri_plugin_window_state::StateFlags::POSITION)
                .build(),
        )
        .manage(Mutex::new(engine))
        .setup(|app| {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tomatick");
}
