use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Pomodoro,
    Countdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineState {
    /// Nothing running, waiting for the user to start.
    Idle,
    Running,
    /// Paused by the user.
    Paused,
    /// Paused by the system (sleep or lock).
    AutoPaused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseKind {
    Work,
    ShortBreak,
    LongBreak,
}

impl PhaseKind {
    pub fn is_break(self) -> bool {
        !matches!(self, PhaseKind::Work)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    /// Ran to its end.
    Completed,
    /// Ended early by the user.
    Skipped,
    /// Abandoned: reset after start, mode switch, quit or crash.
    Interrupted,
}

/// The events that can raise alerts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertEvent {
    WorkEnd,
    ShortBreakEnd,
    LongBreakEnd,
    CountdownEnd,
}

/// One finished phase, ready to be stored in the history.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhaseRecord {
    pub kind: PhaseKind,
    pub started_unix_ms: u64,
    pub ended_unix_ms: u64,
    pub planned_secs: u32,
    pub actual_secs: u32,
    pub paused_secs: u32,
    pub extended_secs: u32,
    pub status: PhaseStatus,
    /// Exercise labels shown during the phase.
    pub labels: Vec<String>,
}

/// Something the rest of the app reacts to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    PhaseStarted {
        kind: PhaseKind,
    },
    /// A long break moved to its next segment.
    SegmentChanged {
        index: u32,
        count: u32,
        label: Option<String>,
    },
    PhaseRecorded(PhaseRecord),
    /// An alert should fire. `label` is the exercise of the break that
    /// follows a work phase.
    Alert {
        event: AlertEvent,
        label: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SegmentInfo {
    pub index: u32,
    pub count: u32,
    pub total_ms: u64,
    pub remaining_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CycleInfo {
    /// Position in the cycle, starting at 1.
    pub position: u32,
    pub length: u32,
}

/// Everything the UI needs to render the current state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub mode: Mode,
    pub state: EngineState,
    /// `None` in countdown mode.
    pub phase: Option<PhaseKind>,
    pub total_ms: u64,
    pub remaining_ms: u64,
    /// Present during a long break.
    pub segment: Option<SegmentInfo>,
    /// Current exercise label, if any.
    pub label: Option<String>,
    pub cycle: CycleInfo,
}
