use super::clock::Clock;
use super::exercises::{Exercise, Rotation};
use super::segments::segment_lengths_secs;
use super::settings::{
    COUNTDOWN_SECS_RANGE, DEFAULT_COUNTDOWN_SECS, EngineError, SleepBehavior, TimerSettings,
    check_range,
};
use super::timer::Timer;
use super::types::{
    AlertEvent, CycleInfo, EngineState, Event, Mode, PhaseKind, PhaseRecord, PhaseStatus,
    SegmentInfo, Snapshot,
};

/// When a phase ends later than this after its scheduled end (the computer
/// was asleep, the app was frozen), the next phase starts "now" instead of at
/// the scheduled end, so a long absence does not eat into it.
const CATCH_UP_TOLERANCE_MS: u64 = 2_000;

fn secs_to_ms(secs: u32) -> u64 {
    u64::from(secs) * 1000
}

fn ms_to_secs(ms: u64) -> u32 {
    u32::try_from((ms + 500) / 1000).unwrap_or(u32::MAX)
}

fn alert_for(kind: PhaseKind) -> AlertEvent {
    match kind {
        PhaseKind::Work => AlertEvent::WorkEnd,
        PhaseKind::ShortBreak => AlertEvent::ShortBreakEnd,
        PhaseKind::LongBreak => AlertEvent::LongBreakEnd,
    }
}

/// State of the Pomodoro mode.
struct Pomodoro {
    kind: PhaseKind,
    timer: Timer,
    /// Work phases completed in the current cycle.
    completed: u32,
    /// One entry per started segment of a break (a short break has one).
    /// Empty during work phases.
    labels: Vec<Option<String>>,
    /// Segment lengths of a long break, empty otherwise.
    segment_lengths_ms: Vec<u64>,
}

/// The timer engine.
///
/// It is a plain state machine: it does no I/O and starts no threads. The
/// host calls [`Engine::tick`] regularly (a few times per second) and handles
/// the returned [`Event`]s. Commands return the events they cause.
///
/// Only one mode is active at a time. Switching mode resets the timer of the
/// mode being left.
pub struct Engine<C: Clock> {
    clock: C,
    settings: TimerSettings,
    mode: Mode,
    pomodoro: Pomodoro,
    countdown_secs: u32,
    countdown: Timer,
    rotation: Rotation,
}

impl<C: Clock> Engine<C> {
    pub fn new(
        clock: C,
        settings: TimerSettings,
        exercises: Vec<Exercise>,
    ) -> Result<Self, EngineError> {
        settings.validate()?;
        let mut engine = Self {
            clock,
            settings,
            mode: Mode::Pomodoro,
            pomodoro: Pomodoro {
                kind: PhaseKind::Work,
                timer: Timer::new(0),
                completed: 0,
                labels: Vec::new(),
                segment_lengths_ms: Vec::new(),
            },
            countdown_secs: DEFAULT_COUNTDOWN_SECS,
            countdown: Timer::new(secs_to_ms(DEFAULT_COUNTDOWN_SECS)),
            rotation: Rotation::new(exercises),
        };
        engine.load_phase(PhaseKind::Work);
        Ok(engine)
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn settings(&self) -> &TimerSettings {
        &self.settings
    }

    pub fn state(&self) -> EngineState {
        self.active_timer().state()
    }

    // ----- commands -----

    /// Starts the current phase, or resumes it if it is paused.
    pub fn start(&mut self) -> Vec<Event> {
        let (now, unix) = self.times();
        let mut events = Vec::new();
        match self.active_timer().state() {
            EngineState::Idle => {
                self.active_timer_mut().start(now, unix);
                if self.mode == Mode::Pomodoro {
                    events.push(Event::PhaseStarted {
                        kind: self.pomodoro.kind,
                    });
                }
            }
            EngineState::Paused | EngineState::AutoPaused => self.active_timer_mut().resume(now),
            EngineState::Running => {}
        }
        events
    }

    /// Pauses a running timer. Does nothing otherwise.
    pub fn pause(&mut self) {
        let now = self.clock.now_ms();
        let timer = self.active_timer_mut();
        if timer.is_running() {
            timer.pause(now, false);
        }
    }

    /// Resumes a paused timer. Does nothing otherwise.
    pub fn resume(&mut self) {
        let now = self.clock.now_ms();
        self.active_timer_mut().resume(now);
    }

    /// Pauses a running timer, or starts or resumes a stopped one.
    pub fn toggle(&mut self) -> Vec<Event> {
        if self.active_timer().is_running() {
            self.pause();
            Vec::new()
        } else {
            self.start()
        }
    }

    /// Ends the current phase now and moves to the next one. The next phase
    /// starts only if auto-start is on. Pomodoro mode only.
    pub fn skip(&mut self) -> Vec<Event> {
        self.skip_inner(false)
    }

    /// Like [`Engine::skip`], but always starts the next phase. Used by the
    /// "skip break" button of the overlay.
    pub fn skip_and_start(&mut self) -> Vec<Event> {
        self.skip_inner(true)
    }

    /// Returns the current phase (or the countdown) to its full duration,
    /// not started. A Pomodoro phase that had started is recorded as
    /// interrupted.
    pub fn reset(&mut self) -> Vec<Event> {
        let (now, unix) = self.times();
        let mut events = Vec::new();
        match self.mode {
            Mode::Pomodoro => {
                if let Some(record) = self.interrupted_record(now, unix) {
                    events.push(Event::PhaseRecorded(record));
                }
                self.reload_current_phase();
            }
            Mode::Countdown => self.countdown = Timer::new(secs_to_ms(self.countdown_secs)),
        }
        events
    }

    /// Adds time to a break that has started. Does nothing otherwise.
    pub fn extend(&mut self, secs: u32) {
        if self.mode == Mode::Pomodoro
            && self.pomodoro.kind.is_break()
            && self.pomodoro.timer.is_started()
        {
            self.pomodoro.timer.extend(secs_to_ms(secs));
        }
    }

    /// Switches mode. The timer of the mode being left is reset: a Pomodoro
    /// phase in progress is recorded as interrupted and the cycle restarts,
    /// a countdown returns to its set duration.
    pub fn set_mode(&mut self, mode: Mode) -> Vec<Event> {
        if self.mode == mode {
            return Vec::new();
        }
        let events = self.reset();
        if self.mode == Mode::Pomodoro {
            self.pomodoro.completed = 0;
            self.load_phase(PhaseKind::Work);
        }
        self.mode = mode;
        events
    }

    /// Sets the countdown duration and resets the countdown.
    pub fn set_countdown(&mut self, secs: u32) -> Result<(), EngineError> {
        check_range("countdown_secs", secs, COUNTDOWN_SECS_RANGE)?;
        self.countdown_secs = secs;
        self.countdown = Timer::new(secs_to_ms(secs));
        Ok(())
    }

    /// Applies new settings. A phase that has not started picks up its new
    /// duration, a phase in progress keeps the one it started with.
    pub fn update_settings(&mut self, settings: TimerSettings) -> Result<(), EngineError> {
        settings.validate()?;
        self.settings = settings;
        if !self.pomodoro.timer.is_started() {
            let kind = self.pomodoro.kind;
            let planned = self.planned_ms(kind);
            self.pomodoro.timer.set_planned(planned);
            if kind == PhaseKind::LongBreak {
                self.pomodoro.segment_lengths_ms = self.long_break_segments_ms();
            }
        }
        Ok(())
    }

    pub fn set_exercises(&mut self, exercises: Vec<Exercise>) {
        self.rotation.set_items(exercises);
    }

    /// Call when the computer goes to sleep or the screen locks. With the
    /// auto-pause behavior a running timer is paused. With "keep running"
    /// nothing happens: the host just keeps calling [`Engine::tick`] after
    /// waking up.
    pub fn on_system_suspend(&mut self) {
        if self.settings.sleep_behavior != SleepBehavior::AutoPause {
            return;
        }
        let now = self.clock.now_ms();
        let timer = self.active_timer_mut();
        if timer.is_running() {
            timer.pause(now, true);
        }
    }

    /// Call when the app is about to quit. A phase in progress is recorded as
    /// interrupted.
    pub fn shutdown(&mut self) -> Vec<Event> {
        self.reset()
    }

    /// Advances the engine to the current time: ends a phase that reached
    /// zero, moves a long break to its next segment, fires alerts.
    pub fn tick(&mut self) -> Vec<Event> {
        let (now, unix) = self.times();
        let mut events = Vec::new();
        match self.mode {
            Mode::Pomodoro => match self.pomodoro.timer.end_instant() {
                Some(end) if now >= end => self.complete_phase(end, now, unix, &mut events),
                Some(_) => self.advance_segments(now, &mut events),
                None => {}
            },
            Mode::Countdown => {
                if self.countdown.end_instant().is_some_and(|end| now >= end) {
                    events.push(Event::Alert {
                        event: AlertEvent::CountdownEnd,
                        label: None,
                    });
                    self.countdown = Timer::new(secs_to_ms(self.countdown_secs));
                }
            }
        }
        events
    }

    /// The state for the UI.
    pub fn snapshot(&self) -> Snapshot {
        let now = self.clock.now_ms();
        let timer = self.active_timer();
        let (phase, segment, label) = match self.mode {
            Mode::Countdown => (None, None, None),
            Mode::Pomodoro => (
                Some(self.pomodoro.kind),
                self.segment_info(now),
                self.current_label(now),
            ),
        };
        Snapshot {
            mode: self.mode,
            state: timer.state(),
            phase,
            total_ms: timer.total_ms(),
            remaining_ms: timer.remaining_ms(now),
            segment,
            label,
            cycle: self.cycle_info(),
        }
    }

    // ----- internals -----

    fn times(&self) -> (u64, u64) {
        (self.clock.now_ms(), self.clock.unix_ms())
    }

    fn active_timer(&self) -> &Timer {
        match self.mode {
            Mode::Pomodoro => &self.pomodoro.timer,
            Mode::Countdown => &self.countdown,
        }
    }

    fn active_timer_mut(&mut self) -> &mut Timer {
        match self.mode {
            Mode::Pomodoro => &mut self.pomodoro.timer,
            Mode::Countdown => &mut self.countdown,
        }
    }

    fn planned_ms(&self, kind: PhaseKind) -> u64 {
        secs_to_ms(match kind {
            PhaseKind::Work => self.settings.work_secs,
            PhaseKind::ShortBreak => self.settings.short_break_secs,
            PhaseKind::LongBreak => self.settings.long_break_secs,
        })
    }

    fn long_break_segments_ms(&self) -> Vec<u64> {
        segment_lengths_secs(
            self.settings.long_break_secs,
            self.settings.short_break_secs,
        )
        .into_iter()
        .map(secs_to_ms)
        .collect()
    }

    /// Loads a fresh, not started phase. Breaks take their first exercise
    /// from the rotation right away so the alert that announces the break
    /// can show it.
    fn load_phase(&mut self, kind: PhaseKind) {
        let planned = self.planned_ms(kind);
        let segment_lengths_ms = if kind == PhaseKind::LongBreak {
            self.long_break_segments_ms()
        } else {
            Vec::new()
        };
        let labels = if kind.is_break() {
            vec![self.rotation.next_label()]
        } else {
            Vec::new()
        };
        self.pomodoro.kind = kind;
        self.pomodoro.timer = Timer::new(planned);
        self.pomodoro.labels = labels;
        self.pomodoro.segment_lengths_ms = segment_lengths_ms;
    }

    /// Restarts the current phase without rolling the exercise rotation back.
    fn reload_current_phase(&mut self) {
        let kind = self.pomodoro.kind;
        self.pomodoro.timer = Timer::new(self.planned_ms(kind));
        self.pomodoro.labels.truncate(1);
        if kind == PhaseKind::LongBreak {
            self.pomodoro.segment_lengths_ms = self.long_break_segments_ms();
        }
    }

    /// Updates the cycle counter and returns the phase that follows `kind`.
    fn advance_cycle(&mut self, kind: PhaseKind, completed: bool) -> PhaseKind {
        match kind {
            PhaseKind::Work => {
                if completed {
                    self.pomodoro.completed += 1;
                }
                if self.pomodoro.completed >= self.settings.sessions_before_long {
                    PhaseKind::LongBreak
                } else {
                    PhaseKind::ShortBreak
                }
            }
            PhaseKind::ShortBreak => PhaseKind::Work,
            PhaseKind::LongBreak => {
                self.pomodoro.completed = 0;
                PhaseKind::Work
            }
        }
    }

    fn first_label(&self) -> Option<String> {
        self.pomodoro.labels.first().cloned().flatten()
    }

    fn complete_phase(&mut self, end: u64, now: u64, unix: u64, events: &mut Vec<Event>) {
        let late_ms = now.saturating_sub(end);
        let kind = self.pomodoro.kind;
        let record = self.build_record(PhaseStatus::Completed, unix.saturating_sub(late_ms), now);
        events.push(Event::PhaseRecorded(record));

        let next = self.advance_cycle(kind, true);
        self.load_phase(next);
        events.push(Event::Alert {
            event: alert_for(kind),
            label: self.first_label(),
        });

        if self.settings.auto_start {
            let start = if late_ms <= CATCH_UP_TOLERANCE_MS {
                end
            } else {
                now
            };
            let start_unix = unix.saturating_sub(now.saturating_sub(start));
            self.pomodoro.timer.start(start, start_unix);
            events.push(Event::PhaseStarted { kind: next });
        }
    }

    fn skip_inner(&mut self, force_start: bool) -> Vec<Event> {
        if self.mode != Mode::Pomodoro {
            return Vec::new();
        }
        let (now, unix) = self.times();
        let mut events = Vec::new();
        let kind = self.pomodoro.kind;
        if self.pomodoro.timer.is_started() {
            let record = self.build_record(PhaseStatus::Skipped, unix, now);
            events.push(Event::PhaseRecorded(record));
        }
        let next = self.advance_cycle(kind, false);
        self.load_phase(next);
        if force_start || self.settings.auto_start {
            self.pomodoro.timer.start(now, unix);
            events.push(Event::PhaseStarted { kind: next });
        }
        events
    }

    fn interrupted_record(&self, now: u64, unix: u64) -> Option<PhaseRecord> {
        self.pomodoro
            .timer
            .is_started()
            .then(|| self.build_record(PhaseStatus::Interrupted, unix, now))
    }

    fn build_record(&self, status: PhaseStatus, ended_unix_ms: u64, now: u64) -> PhaseRecord {
        let timer = &self.pomodoro.timer;
        let actual_ms = if status == PhaseStatus::Completed {
            timer.total_ms()
        } else {
            timer.run_elapsed_ms(now)
        };
        PhaseRecord {
            kind: self.pomodoro.kind,
            started_unix_ms: timer.started_unix_ms().unwrap_or(ended_unix_ms),
            ended_unix_ms,
            planned_secs: ms_to_secs(timer.planned_ms()),
            actual_secs: ms_to_secs(actual_ms),
            paused_secs: ms_to_secs(timer.paused_ms(now)),
            extended_secs: ms_to_secs(timer.extended_ms()),
            status,
            labels: self.pomodoro.labels.iter().flatten().cloned().collect(),
        }
    }

    /// Index of the long break segment at a given run time. The last segment
    /// absorbs any extension.
    fn segment_index(&self, elapsed_ms: u64) -> usize {
        let lengths = &self.pomodoro.segment_lengths_ms;
        let mut end = 0;
        for (index, length) in lengths.iter().enumerate() {
            end += length;
            if elapsed_ms < end {
                return index;
            }
        }
        lengths.len().saturating_sub(1)
    }

    /// Starts the segments of a long break that the run time has reached.
    fn advance_segments(&mut self, now: u64, events: &mut Vec<Event>) {
        if self.pomodoro.kind != PhaseKind::LongBreak {
            return;
        }
        let index = self.segment_index(self.pomodoro.timer.run_elapsed_ms(now));
        let count = self.pomodoro.segment_lengths_ms.len() as u32;
        while self.pomodoro.labels.len() <= index {
            let label = self.rotation.next_label();
            self.pomodoro.labels.push(label.clone());
            events.push(Event::SegmentChanged {
                index: (self.pomodoro.labels.len() - 1) as u32,
                count,
                label,
            });
        }
    }

    fn segment_info(&self, now: u64) -> Option<SegmentInfo> {
        let lengths = &self.pomodoro.segment_lengths_ms;
        if self.pomodoro.kind != PhaseKind::LongBreak || lengths.is_empty() {
            return None;
        }
        let timer = &self.pomodoro.timer;
        let elapsed = timer.run_elapsed_ms(now);
        let index = self.segment_index(elapsed);
        let start: u64 = lengths[..index].iter().sum();
        let end = if index + 1 == lengths.len() {
            timer.total_ms()
        } else {
            start + lengths[index]
        };
        Some(SegmentInfo {
            index: index as u32,
            count: lengths.len() as u32,
            total_ms: end.saturating_sub(start),
            remaining_ms: end.saturating_sub(elapsed),
        })
    }

    fn current_label(&self, now: u64) -> Option<String> {
        let labels = &self.pomodoro.labels;
        match self.pomodoro.kind {
            PhaseKind::Work => None,
            PhaseKind::ShortBreak => labels.first().cloned().flatten(),
            PhaseKind::LongBreak => {
                let index = self.segment_index(self.pomodoro.timer.run_elapsed_ms(now));
                // Hold the previous label until the next tick starts the new
                // segment.
                let index = index.min(labels.len().saturating_sub(1));
                labels.get(index).cloned().flatten()
            }
        }
    }

    fn cycle_info(&self) -> CycleInfo {
        let length = self.settings.sessions_before_long;
        let completed = self.pomodoro.completed;
        let position = match self.pomodoro.kind {
            PhaseKind::Work => completed + 1,
            PhaseKind::ShortBreak | PhaseKind::LongBreak => completed.max(1),
        };
        CycleInfo {
            position: position.min(length),
            length,
        }
    }
}
