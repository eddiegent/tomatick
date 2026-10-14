//! Local persistence in SQLite (SPEC.md section 14).
//!
//! This module does not depend on Tauri. The schema is versioned with
//! `PRAGMA user_version`: each entry of [`MIGRATIONS`] upgrades the database
//! by one version and runs once, inside a transaction. Never edit a migration
//! that has shipped, add a new one.

use std::fmt;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::alerts::{AlertError, AlertSettings};
use crate::engine::{
    COUNTDOWN_SECS_RANGE, DEFAULT_COUNTDOWN_SECS, EngineError, Exercise, TimerSettings,
};
use crate::i18n;
use crate::shortcuts::{ShortcutError, ShortcutSettings};

/// The key under which [`AppSettings`] is stored in the `settings` table.
const SETTINGS_KEY: &str = "app";
/// The key under which the position in the exercise rotation is stored.
const ROTATION_KEY: &str = "rotation";

/// The countdown presets of a fresh install: 5, 10, 15 and 30 minutes.
pub const DEFAULT_COUNTDOWN_PRESETS: [u32; 4] = [5 * 60, 10 * 60, 15 * 60, 30 * 60];
/// The daily goal of a fresh install, in pomodoros, and what it can be set to.
pub const DEFAULT_DAILY_GOAL: u32 = 8;
pub const DAILY_GOAL_RANGE: (u32, u32) = (1, 50);
/// How many countdown presets can be kept.
pub const COUNTDOWN_PRESETS_COUNT: (u32, u32) = (1, 6);

/// Longest exercise name or amount that is kept.
const MAX_TEXT_CHARS: usize = 60;
/// Most exercises the list can hold.
const MAX_EXERCISES: usize = 50;

const MIGRATIONS: &[&str] = &[
    // Version 1: settings and the break exercise list.
    "CREATE TABLE settings (
         key   TEXT PRIMARY KEY NOT NULL,
         value TEXT NOT NULL
     );
     CREATE TABLE exercises (
         position INTEGER PRIMARY KEY NOT NULL,
         name     TEXT NOT NULL,
         amount   TEXT
     );",
    // Version 2: the history of the phases (SPEC.md section 12).
    "CREATE TABLE phases (
         id               INTEGER PRIMARY KEY,
         kind             TEXT NOT NULL CHECK (kind IN ('work', 'short_break', 'long_break')),
         started_at       TEXT NOT NULL,
         ended_at         TEXT,
         planned_seconds  INTEGER NOT NULL,
         actual_seconds   INTEGER NOT NULL DEFAULT 0,
         paused_seconds   INTEGER NOT NULL DEFAULT 0,
         extended_seconds INTEGER NOT NULL DEFAULT 0,
         status           TEXT NOT NULL CHECK (status IN ('completed', 'skipped', 'interrupted')),
         labels           TEXT
     );
     CREATE INDEX phases_started_at ON phases (started_at);
     CREATE INDEX phases_kind_status ON phases (kind, status);",
];

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    Json(serde_json::Error),
    Invalid(SettingsError),
    /// A date that is not `YYYY-MM-DD`.
    BadDate(String),
    /// The database was written by a newer version of the app.
    TooNew {
        found: usize,
        supported: usize,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::Database(error) => write!(f, "database error: {error}"),
            StorageError::Json(error) => write!(f, "invalid stored settings: {error}"),
            StorageError::Invalid(error) => write!(f, "{error}"),
            StorageError::BadDate(text) => write!(f, "not a date (YYYY-MM-DD): {text}"),
            StorageError::TooNew { found, supported } => write!(
                f,
                "the database is at version {found} but this app supports up to {supported}"
            ),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rusqlite::Error> for StorageError {
    fn from(error: rusqlite::Error) -> Self {
        StorageError::Database(error)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(error: serde_json::Error) -> Self {
        StorageError::Json(error)
    }
}

impl From<SettingsError> for StorageError {
    fn from(error: SettingsError) -> Self {
        StorageError::Invalid(error)
    }
}

/// A setting that is out of range or otherwise not allowed.
#[derive(Debug, PartialEq, Eq)]
pub enum SettingsError {
    Timer(EngineError),
    Alerts(AlertError),
    Shortcuts(ShortcutError),
    /// A language that has no locale file.
    Language(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Timer(error) => write!(f, "{error}"),
            SettingsError::Alerts(error) => write!(f, "{error}"),
            SettingsError::Shortcuts(error) => write!(f, "{error}"),
            SettingsError::Language(code) => write!(f, "unknown language: {code}"),
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<EngineError> for SettingsError {
    fn from(error: EngineError) -> Self {
        SettingsError::Timer(error)
    }
}

impl From<ShortcutError> for SettingsError {
    fn from(error: ShortcutError) -> Self {
        SettingsError::Shortcuts(error)
    }
}

impl From<AlertError> for SettingsError {
    fn from(error: AlertError) -> Self {
        SettingsError::Alerts(error)
    }
}

/// The three widget sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WidgetSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Everything the settings dialog edits, except the exercise list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub timer: TimerSettings,
    pub widget_size: WidgetSize,
    /// Starting value of the countdown timer.
    pub countdown_secs: u32,
    /// The quick-pick durations shown when the countdown is edited, in seconds.
    pub countdown_presets: Vec<u32>,
    /// Pomodoros to reach each day, shown in the statistics.
    pub daily_goal: u32,
    /// Start Tomatick when the user logs in.
    pub launch_at_login: bool,
    /// Look for a newer release on launch and then every day.
    pub check_for_updates: bool,
    pub shortcuts: ShortcutSettings,
    /// "system", or the code of a language that has a locale file.
    pub language: String,
    pub alerts: AlertSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            timer: TimerSettings::classic(),
            widget_size: WidgetSize::Medium,
            countdown_secs: DEFAULT_COUNTDOWN_SECS,
            countdown_presets: DEFAULT_COUNTDOWN_PRESETS.to_vec(),
            daily_goal: DEFAULT_DAILY_GOAL,
            launch_at_login: false,
            check_for_updates: true,
            shortcuts: ShortcutSettings::default(),
            language: i18n::SYSTEM.to_string(),
            alerts: AlertSettings::default(),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), SettingsError> {
        self.timer.validate()?;
        crate::engine::check_range("countdown_secs", self.countdown_secs, COUNTDOWN_SECS_RANGE)?;
        crate::engine::check_range(
            "countdown_presets (how many)",
            self.countdown_presets.len() as u32,
            COUNTDOWN_PRESETS_COUNT,
        )?;
        for preset in &self.countdown_presets {
            crate::engine::check_range("countdown_presets", *preset, COUNTDOWN_SECS_RANGE)?;
        }
        crate::engine::check_range("daily_goal", self.daily_goal, DAILY_GOAL_RANGE)?;
        self.shortcuts.validate()?;
        if self.language != i18n::SYSTEM && !i18n::is_supported(&self.language) {
            return Err(SettingsError::Language(self.language.clone()));
        }
        self.alerts.validate()?;
        Ok(())
    }
}

/// Trims exercise names and amounts, drops entries without a name, and caps
/// the length of the list and of each text.
pub fn clean_exercises(exercises: Vec<Exercise>) -> Vec<Exercise> {
    let cap = |text: &str| text.trim().chars().take(MAX_TEXT_CHARS).collect::<String>();
    exercises
        .into_iter()
        .filter_map(|exercise| {
            let name = cap(&exercise.name);
            if name.is_empty() {
                return None;
            }
            let amount = exercise
                .amount
                .as_deref()
                .map(cap)
                .filter(|amount| !amount.is_empty());
            Some(Exercise { name, amount })
        })
        .take(MAX_EXERCISES)
        .collect()
}

pub struct Store {
    pub(crate) connection: Connection,
}

impl Store {
    /// Opens (and creates or upgrades) the database file.
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self, StorageError> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(connection: Connection) -> Result<Self, StorageError> {
        let mut store = Self { connection };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<(), StorageError> {
        let current: usize = self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?;
        if current > MIGRATIONS.len() {
            return Err(StorageError::TooNew {
                found: current,
                supported: MIGRATIONS.len(),
            });
        }
        for (index, sql) in MIGRATIONS.iter().enumerate().skip(current) {
            let transaction = self.connection.transaction()?;
            transaction.execute_batch(sql)?;
            // PRAGMA does not accept bound parameters.
            transaction.execute_batch(&format!("PRAGMA user_version = {}", index + 1))?;
            transaction.commit()?;
        }
        Ok(())
    }

    pub fn schema_version(&self) -> Result<usize, StorageError> {
        Ok(self
            .connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))?)
    }

    /// The stored settings, or the classic defaults on first run. Stored
    /// values that no longer validate fall back to the defaults too, so a
    /// hand-edited database cannot stop the app from starting.
    pub fn load_settings(&self) -> Result<AppSettings, StorageError> {
        let stored: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![SETTINGS_KEY],
                |row| row.get(0),
            )
            .optional()?;
        let Some(json) = stored else {
            return Ok(AppSettings::default());
        };
        match serde_json::from_str::<AppSettings>(&json) {
            Ok(settings) if settings.validate().is_ok() => Ok(settings),
            _ => Ok(AppSettings::default()),
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
        settings.validate()?;
        let json = serde_json::to_string(settings)?;
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![SETTINGS_KEY, json],
        )?;
        Ok(())
    }

    /// Where the exercise rotation stood when the app last ran, or 0.
    pub fn load_rotation(&self) -> Result<usize, StorageError> {
        let stored: Option<String> = self
            .connection
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![ROTATION_KEY],
                |row| row.get(0),
            )
            .optional()?;
        Ok(stored.and_then(|text| text.parse().ok()).unwrap_or(0))
    }

    /// Remembers the rotation position. Writes nothing when it is unchanged.
    pub fn save_rotation(&self, position: usize) -> Result<(), StorageError> {
        self.connection.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value
             WHERE value != excluded.value",
            params![ROTATION_KEY, position.to_string()],
        )?;
        Ok(())
    }

    pub fn load_exercises(&self) -> Result<Vec<Exercise>, StorageError> {
        let mut statement = self
            .connection
            .prepare("SELECT name, amount FROM exercises ORDER BY position")?;
        let rows = statement.query_map([], |row| {
            Ok(Exercise {
                name: row.get(0)?,
                amount: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    /// Replaces the whole list, keeping the given order. Returns what was
    /// actually stored (after cleaning).
    pub fn save_exercises(
        &mut self,
        exercises: Vec<Exercise>,
    ) -> Result<Vec<Exercise>, StorageError> {
        let exercises = clean_exercises(exercises);
        let transaction = self.connection.transaction()?;
        transaction.execute("DELETE FROM exercises", [])?;
        for (position, exercise) in exercises.iter().enumerate() {
            transaction.execute(
                "INSERT INTO exercises (position, name, amount) VALUES (?1, ?2, ?3)",
                params![position as i64, exercise.name, exercise.amount],
            )?;
        }
        transaction.commit()?;
        Ok(exercises)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::SoundChoice;

    fn exercise(name: &str, amount: Option<&str>) -> Exercise {
        Exercise {
            name: name.to_string(),
            amount: amount.map(str::to_string),
        }
    }

    #[test]
    fn first_run_gives_classic_defaults() {
        let store = Store::open_in_memory().unwrap();
        let settings = store.load_settings().unwrap();
        assert_eq!(settings, AppSettings::default());
        assert_eq!(settings.timer, TimerSettings::classic());
        assert_eq!(settings.countdown_secs, 600);
        assert!(store.load_exercises().unwrap().is_empty());
    }

    #[test]
    fn rotation_position_round_trips_and_defaults_to_zero() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.load_rotation().unwrap(), 0);
        store.save_rotation(3).unwrap();
        assert_eq!(store.load_rotation().unwrap(), 3);
        store.save_rotation(3).unwrap();
        store.save_rotation(1).unwrap();
        assert_eq!(store.load_rotation().unwrap(), 1);
    }

    #[test]
    fn rotation_position_does_not_disturb_the_settings() {
        let store = Store::open_in_memory().unwrap();
        let settings = AppSettings::default();
        store.save_settings(&settings).unwrap();
        store.save_rotation(2).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);
    }

    #[test]
    fn the_daily_goal_defaults_to_8_and_is_validated() {
        let mut settings = AppSettings::default();
        assert_eq!(settings.daily_goal, 8);
        settings.daily_goal = 0;
        assert!(settings.validate().is_err());
        settings.daily_goal = 51;
        assert!(settings.validate().is_err());
        settings.daily_goal = 50;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn desktop_settings_default_and_old_databases_still_load() {
        let settings = AppSettings::default();
        assert!(!settings.launch_at_login);
        assert_eq!(settings.shortcuts.start_pause, "CommandOrControl+Alt+P");

        // Settings stored before these fields existed fall back to defaults.
        let store = Store::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', '{\"daily_goal\": 3}')",
                [],
            )
            .unwrap();
        let loaded = store.load_settings().unwrap();
        assert_eq!(loaded.daily_goal, 3);
        assert_eq!(loaded.shortcuts, ShortcutSettings::default());
        assert!(!loaded.launch_at_login);
    }

    #[test]
    fn the_update_check_is_on_by_default_and_the_choice_is_kept() {
        assert!(AppSettings::default().check_for_updates);

        // Settings stored before this field existed get the default.
        let store = Store::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', '{\"daily_goal\": 3}')",
                [],
            )
            .unwrap();
        assert!(store.load_settings().unwrap().check_for_updates);

        let settings = AppSettings {
            check_for_updates: false,
            ..AppSettings::default()
        };
        store.save_settings(&settings).unwrap();
        assert!(!store.load_settings().unwrap().check_for_updates);
    }

    #[test]
    fn the_language_defaults_to_the_system_and_must_exist() {
        let mut settings = AppSettings::default();
        assert_eq!(settings.language, "system");
        settings.language = "fr".to_string();
        assert!(settings.validate().is_ok());
        settings.language = "xx".to_string();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn conflicting_shortcuts_are_refused_by_validation() {
        let mut settings = AppSettings::default();
        settings.shortcuts.skip = settings.shortcuts.start_pause.clone();
        assert!(settings.validate().is_err());
    }

    #[test]
    fn countdown_presets_default_to_5_10_15_and_30_minutes() {
        let settings = AppSettings::default();
        assert_eq!(settings.countdown_presets, vec![300, 600, 900, 1800]);
    }

    #[test]
    fn settings_saved_before_presets_existed_get_the_default_presets() {
        let store = Store::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', '{\"countdown_secs\": 90}')",
                [],
            )
            .unwrap();
        let settings = store.load_settings().unwrap();
        assert_eq!(settings.countdown_secs, 90);
        assert_eq!(settings.countdown_presets, vec![300, 600, 900, 1800]);
    }

    #[test]
    fn countdown_presets_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let settings = AppSettings {
            countdown_presets: vec![90, 20 * 60],
            ..AppSettings::default()
        };
        store.save_settings(&settings).unwrap();
        assert_eq!(
            store.load_settings().unwrap().countdown_presets,
            vec![90, 1200]
        );
    }

    #[test]
    fn countdown_presets_are_validated() {
        let with = |countdown_presets: Vec<u32>| AppSettings {
            countdown_presets,
            ..AppSettings::default()
        };
        assert!(with(Vec::new()).validate().is_err(), "at least one preset");
        assert!(with(vec![60; 7]).validate().is_err(), "at most six presets");
        assert!(
            with(vec![60, 0]).validate().is_err(),
            "a preset lasts a second"
        );
        assert!(
            with(vec![24 * 60 * 60]).validate().is_err(),
            "a preset stays under 24 hours"
        );
        assert!(with(vec![24 * 60 * 60 - 1]).validate().is_ok());
    }

    #[test]
    fn migrations_set_the_schema_version() {
        let store = Store::open_in_memory().unwrap();
        assert_eq!(store.schema_version().unwrap(), MIGRATIONS.len());
    }

    #[test]
    fn settings_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let mut settings = AppSettings::default();
        settings.timer.work_secs = 50 * 60;
        settings.timer.auto_start = true;
        settings.widget_size = WidgetSize::Large;
        settings.countdown_secs = 90;
        store.save_settings(&settings).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);

        settings.countdown_secs = 120;
        store.save_settings(&settings).unwrap();
        assert_eq!(store.load_settings().unwrap().countdown_secs, 120);
    }

    #[test]
    fn invalid_settings_are_refused_and_not_stored() {
        let store = Store::open_in_memory().unwrap();
        let mut settings = AppSettings::default();
        settings.timer.work_secs = 5;
        assert!(store.save_settings(&settings).is_err());
        assert_eq!(store.load_settings().unwrap(), AppSettings::default());

        settings = AppSettings::default();
        settings.countdown_secs = 0;
        assert!(store.save_settings(&settings).is_err());
    }

    #[test]
    fn corrupt_stored_settings_fall_back_to_defaults() {
        let store = Store::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', 'not json')",
                [],
            )
            .unwrap();
        assert_eq!(store.load_settings().unwrap(), AppSettings::default());

        store
            .connection
            .execute(
                "UPDATE settings SET value = '{\"countdown_secs\": 0}' WHERE key = 'app'",
                [],
            )
            .unwrap();
        assert_eq!(store.load_settings().unwrap(), AppSettings::default());
    }

    #[test]
    fn alert_settings_round_trip() {
        let store = Store::open_in_memory().unwrap();
        let mut settings = AppSettings::default();
        settings.alerts.volume = 35;
        settings.alerts.muted = true;
        settings.alerts.work_end.notification = false;
        settings.alerts.work_end.sound = SoundChoice::Custom("gong.mp3".to_string());
        settings.alerts.countdown_end.bring_to_front = true;
        store.save_settings(&settings).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);
    }

    #[test]
    fn settings_saved_before_alerts_existed_get_the_alert_defaults() {
        let store = Store::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', '{\"countdown_secs\": 90}')",
                [],
            )
            .unwrap();
        let settings = store.load_settings().unwrap();
        assert_eq!(settings.countdown_secs, 90);
        assert_eq!(settings.alerts, AlertSettings::default());
    }

    #[test]
    fn invalid_alert_settings_are_refused_and_not_stored() {
        let store = Store::open_in_memory().unwrap();
        let mut settings = AppSettings::default();
        settings.alerts.volume = 150;
        assert!(store.save_settings(&settings).is_err());

        settings = AppSettings::default();
        settings.alerts.short_break_end.sound = SoundChoice::Custom("../x.mp3".to_string());
        assert!(store.save_settings(&settings).is_err());
        assert_eq!(store.load_settings().unwrap(), AppSettings::default());
    }

    #[test]
    fn missing_fields_use_defaults() {
        let store = Store::open_in_memory().unwrap();
        store
            .connection
            .execute(
                "INSERT INTO settings (key, value) VALUES ('app', '{\"widget_size\": \"small\"}')",
                [],
            )
            .unwrap();
        let settings = store.load_settings().unwrap();
        assert_eq!(settings.widget_size, WidgetSize::Small);
        assert_eq!(settings.timer, TimerSettings::classic());
    }

    #[test]
    fn exercises_keep_their_order() {
        let mut store = Store::open_in_memory().unwrap();
        let list = vec![
            exercise("Planks", Some("30 s")),
            exercise("Sit-ups", Some("15")),
            exercise("Stretch", None),
        ];
        store.save_exercises(list.clone()).unwrap();
        assert_eq!(store.load_exercises().unwrap(), list);

        let reordered = vec![list[2].clone(), list[0].clone()];
        store.save_exercises(reordered.clone()).unwrap();
        assert_eq!(store.load_exercises().unwrap(), reordered);
    }

    #[test]
    fn exercises_are_cleaned() {
        let mut store = Store::open_in_memory().unwrap();
        let saved = store
            .save_exercises(vec![
                exercise("  Planks  ", Some("  ")),
                exercise("   ", Some("10")),
                exercise("Squats", Some(" 20 ")),
            ])
            .unwrap();
        assert_eq!(
            saved,
            vec![exercise("Planks", None), exercise("Squats", Some("20"))]
        );
        assert_eq!(store.load_exercises().unwrap(), saved);
    }

    #[test]
    fn exercise_list_and_text_are_capped() {
        let long = "x".repeat(200);
        let many: Vec<Exercise> = (0..80).map(|i| exercise(&format!("E{i}"), None)).collect();
        assert_eq!(clean_exercises(many).len(), MAX_EXERCISES);
        let cleaned = clean_exercises(vec![exercise(&long, Some(&long))]);
        assert_eq!(cleaned[0].name.chars().count(), MAX_TEXT_CHARS);
        assert_eq!(
            cleaned[0].amount.as_ref().unwrap().chars().count(),
            MAX_TEXT_CHARS
        );
    }

    #[test]
    fn data_survives_reopening_a_file() {
        let dir = std::env::temp_dir().join(format!("tomatick-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tomatick.db");
        let _ = std::fs::remove_file(&path);

        let mut settings = AppSettings::default();
        settings.timer.short_break_secs = 7 * 60;
        {
            let mut store = Store::open(&path).unwrap();
            store.save_settings(&settings).unwrap();
            store
                .save_exercises(vec![exercise("Planks", None)])
                .unwrap();
        }
        let store = Store::open(&path).unwrap();
        assert_eq!(store.load_settings().unwrap(), settings);
        assert_eq!(store.load_exercises().unwrap().len(), 1);
        drop(store);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_newer_database_is_refused() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(&format!("PRAGMA user_version = {}", MIGRATIONS.len() + 1))
            .unwrap();
        assert!(matches!(
            Store::init(connection),
            Err(StorageError::TooNew { .. })
        ));
    }
}
