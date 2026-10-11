use super::types::EngineState;

/// A single stopwatch-style countdown with pause accounting.
///
/// All times are milliseconds on the engine clock. The timer never reads a
/// clock itself: the caller passes `now` in.
#[derive(Clone, Debug)]
pub(crate) struct Timer {
    planned_ms: u64,
    extended_ms: u64,
    /// Run time accumulated before the current run span.
    run_before_ms: u64,
    /// Start of the current run span, `None` while not running.
    run_since: Option<u64>,
    /// Pause time accumulated before the current pause.
    paused_before_ms: u64,
    /// Start of the current pause, `None` while not paused.
    paused_since: Option<u64>,
    auto_paused: bool,
    started: bool,
    started_unix_ms: Option<u64>,
}

impl Timer {
    pub fn new(planned_ms: u64) -> Self {
        Self {
            planned_ms,
            extended_ms: 0,
            run_before_ms: 0,
            run_since: None,
            paused_before_ms: 0,
            paused_since: None,
            auto_paused: false,
            started: false,
            started_unix_ms: None,
        }
    }

    pub fn planned_ms(&self) -> u64 {
        self.planned_ms
    }

    pub fn extended_ms(&self) -> u64 {
        self.extended_ms
    }

    pub fn total_ms(&self) -> u64 {
        self.planned_ms + self.extended_ms
    }

    pub fn started_unix_ms(&self) -> Option<u64> {
        self.started_unix_ms
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn is_running(&self) -> bool {
        self.run_since.is_some()
    }

    pub fn state(&self) -> EngineState {
        if self.run_since.is_some() {
            EngineState::Running
        } else if !self.started {
            EngineState::Idle
        } else if self.auto_paused {
            EngineState::AutoPaused
        } else {
            EngineState::Paused
        }
    }

    /// Run time so far, capped at the total.
    pub fn run_elapsed_ms(&self, now: u64) -> u64 {
        let span = self.run_since.map_or(0, |s| now.saturating_sub(s));
        (self.run_before_ms + span).min(self.total_ms())
    }

    pub fn remaining_ms(&self, now: u64) -> u64 {
        self.total_ms().saturating_sub(self.run_elapsed_ms(now))
    }

    /// Total pause time, including a pause still in progress.
    pub fn paused_ms(&self, now: u64) -> u64 {
        self.paused_before_ms + self.paused_since.map_or(0, |s| now.saturating_sub(s))
    }

    /// The instant the timer reaches zero if it keeps running, `None` while
    /// it is not running.
    pub fn end_instant(&self) -> Option<u64> {
        self.run_since
            .map(|s| s + self.total_ms().saturating_sub(self.run_before_ms))
    }

    pub fn start(&mut self, now: u64, unix_ms: u64) {
        self.started = true;
        self.started_unix_ms = Some(unix_ms);
        self.run_since = Some(now);
    }

    pub fn pause(&mut self, now: u64, auto: bool) {
        if let Some(since) = self.run_since.take() {
            self.run_before_ms += now.saturating_sub(since);
            self.paused_since = Some(now);
            self.auto_paused = auto;
        }
    }

    pub fn resume(&mut self, now: u64) {
        if let Some(since) = self.paused_since.take() {
            self.paused_before_ms += now.saturating_sub(since);
            self.auto_paused = false;
            self.run_since = Some(now);
        }
    }

    pub fn extend(&mut self, ms: u64) {
        self.extended_ms += ms;
    }

    /// Changes the planned duration. Meant for a timer that has not started.
    pub fn set_planned(&mut self, ms: u64) {
        self.planned_ms = ms;
    }
}
