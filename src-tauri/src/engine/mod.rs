//! The timer engine: a pure state machine with no I/O and no threads.
//!
//! See SPEC.md sections 5 to 7. The host (the Tauri layer) calls
//! [`Engine::tick`] a few times per second and forwards the returned
//! [`Event`]s. All time comes from an injectable [`Clock`], so the cycle
//! logic is unit tested without waiting.

mod clock;
mod exercises;
mod machine;
mod segments;
mod settings;
mod timer;
mod types;

#[cfg(test)]
mod tests;

pub use clock::{Clock, SystemClock};
pub use exercises::{Exercise, Rotation};
pub use machine::Engine;
pub use segments::segment_lengths_secs;
pub use settings::{
    COUNTDOWN_SECS_RANGE, DEFAULT_COUNTDOWN_SECS, EngineError, LONG_BREAK_SECS_RANGE,
    SESSIONS_RANGE, SHORT_BREAK_SECS_RANGE, SleepBehavior, TimerSettings, WORK_SECS_RANGE,
};
pub use types::{
    AlertEvent, CycleInfo, EngineState, Event, Mode, PhaseKind, PhaseRecord, PhaseStatus,
    SegmentInfo, Snapshot,
};
