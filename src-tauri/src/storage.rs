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

use crate::engine::{
    COUNTDOWN_SECS_RANGE, DEFAULT_COUNTDOWN_SECS, EngineError, Exercise, TimerSettings,
};

/// The key under which [`AppSettings`] is stored in the `settings` table.
const SETTINGS_KEY: &str = "app";

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
];

#[derive(Debug)]
pub enum StorageError {
    Database(rusqlite::Error),
    Json(serde_json::Error),
    Invalid(EngineError),
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

impl From<EngineError> for StorageError {
    fn from(error: EngineError) -> Self {
        StorageError::Invalid(error)
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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            timer: TimerSettings::classic(),
            widget_size: WidgetSize::Medium,
            countdown_secs: DEFAULT_COUNTDOWN_SECS,
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), EngineError> {
        self.timer.validate()?;
        crate::engine::check_range("countdown_secs", self.countdown_secs, COUNTDOWN_SECS_RANGE)
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
    connection: Connection,
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
