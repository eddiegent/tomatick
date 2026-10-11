use std::fmt;

use serde::{Deserialize, Serialize};

pub const WORK_SECS_RANGE: (u32, u32) = (60, 180 * 60);
pub const SHORT_BREAK_SECS_RANGE: (u32, u32) = (60, 60 * 60);
pub const LONG_BREAK_SECS_RANGE: (u32, u32) = (60, 120 * 60);
pub const SESSIONS_RANGE: (u32, u32) = (2, 12);
pub const COUNTDOWN_SECS_RANGE: (u32, u32) = (1, 24 * 60 * 60 - 1);

pub const DEFAULT_COUNTDOWN_SECS: u32 = 10 * 60;

/// A value outside its allowed range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EngineError {
    OutOfRange {
        field: &'static str,
        min: u32,
        max: u32,
    },
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::OutOfRange { field, min, max } => {
                write!(f, "{field} must be between {min} and {max}")
            }
        }
    }
}

impl std::error::Error for EngineError {}

pub(crate) fn check_range(
    field: &'static str,
    value: u32,
    (min, max): (u32, u32),
) -> Result<(), EngineError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(EngineError::OutOfRange { field, min, max })
    }
}

/// What the timer does when the computer sleeps or the screen locks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SleepBehavior {
    /// Pause on sleep or lock and wait for the user to resume.
    AutoPause,
    /// Follow the real clock. A phase may end while the user is away.
    KeepRunning,
}

/// Timer settings used by the engine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerSettings {
    pub work_secs: u32,
    pub short_break_secs: u32,
    pub long_break_secs: u32,
    /// Work phases before a long break.
    pub sessions_before_long: u32,
    /// Start the next phase automatically when a phase ends.
    pub auto_start: bool,
    pub sleep_behavior: SleepBehavior,
}

impl TimerSettings {
    /// The classic Pomodoro profile: 25 minutes of work, 5 minutes of short
    /// break, a 15 minute long break after 4 work phases.
    pub fn classic() -> Self {
        Self {
            work_secs: 25 * 60,
            short_break_secs: 5 * 60,
            long_break_secs: 15 * 60,
            sessions_before_long: 4,
            auto_start: false,
            sleep_behavior: SleepBehavior::AutoPause,
        }
    }

    pub fn validate(&self) -> Result<(), EngineError> {
        check_range("work_secs", self.work_secs, WORK_SECS_RANGE)?;
        check_range(
            "short_break_secs",
            self.short_break_secs,
            SHORT_BREAK_SECS_RANGE,
        )?;
        check_range(
            "long_break_secs",
            self.long_break_secs,
            LONG_BREAK_SECS_RANGE,
        )?;
        check_range(
            "sessions_before_long",
            self.sessions_before_long,
            SESSIONS_RANGE,
        )
    }
}

impl Default for TimerSettings {
    fn default() -> Self {
        Self::classic()
    }
}
