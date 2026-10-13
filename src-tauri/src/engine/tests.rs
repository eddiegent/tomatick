use std::cell::Cell;
use std::rc::Rc;

use super::*;

const T0: u64 = 1_700_000_000_000;

#[derive(Clone)]
struct TestClock(Rc<Cell<u64>>);

impl Clock for TestClock {
    fn now_ms(&self) -> u64 {
        self.0.get()
    }

    fn unix_ms(&self) -> u64 {
        T0 + self.0.get()
    }
}

struct Harness {
    engine: Engine<TestClock>,
    time: Rc<Cell<u64>>,
}

impl Harness {
    fn new(settings: TimerSettings) -> Self {
        Self::with_exercises(settings, Vec::new())
    }

    fn with_exercises(settings: TimerSettings, exercises: Vec<Exercise>) -> Self {
        let time = Rc::new(Cell::new(0));
        let engine = Engine::new(TestClock(time.clone()), settings, exercises)
            .expect("test settings are valid");
        Self { engine, time }
    }

    fn advance(&self, secs: u64) {
        self.time.set(self.time.get() + secs * 1000);
    }

    /// Moves the clock forward and ticks.
    fn run(&mut self, secs: u64) -> Vec<Event> {
        self.advance(secs);
        self.engine.tick()
    }

    /// Starts the current phase and lets it run to its end.
    fn complete(&mut self, secs: u64) -> Vec<Event> {
        self.engine.start();
        self.run(secs)
    }

    fn snap(&self) -> Snapshot {
        self.engine.snapshot()
    }
}

fn classic() -> TimerSettings {
    TimerSettings::classic()
}

fn exercise(name: &str, amount: Option<&str>) -> Exercise {
    Exercise {
        name: name.to_string(),
        amount: amount.map(str::to_string),
    }
}

fn records(events: &[Event]) -> Vec<PhaseRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::PhaseRecorded(record) => Some(record.clone()),
            _ => None,
        })
        .collect()
}

fn alerts(events: &[Event]) -> Vec<AlertEvent> {
    events
        .iter()
        .filter_map(|event| match event {
            Event::Alert { event, .. } => Some(*event),
            _ => None,
        })
        .collect()
}

fn alert_label(events: &[Event]) -> Option<String> {
    events.iter().find_map(|event| match event {
        Event::Alert { label, .. } => label.clone(),
        _ => None,
    })
}

// ----- settings -----

#[test]
fn classic_profile_matches_the_spec() {
    let s = TimerSettings::classic();
    assert_eq!(s.work_secs, 25 * 60);
    assert_eq!(s.short_break_secs, 5 * 60);
    assert_eq!(s.long_break_secs, 15 * 60);
    assert_eq!(s.sessions_before_long, 4);
    assert!(!s.auto_start);
    assert_eq!(s.sleep_behavior, SleepBehavior::AutoPause);
    assert_eq!(s, TimerSettings::default());
    assert!(s.validate().is_ok());
}

#[test]
fn settings_outside_their_range_are_rejected() {
    let too_short = TimerSettings {
        work_secs: 30,
        ..classic()
    };
    assert_eq!(
        too_short.validate(),
        Err(EngineError::OutOfRange {
            field: "work_secs",
            min: 60,
            max: 180 * 60
        })
    );
    let bad_cycle = TimerSettings {
        sessions_before_long: 1,
        ..classic()
    };
    assert!(bad_cycle.validate().is_err());
    let clock = TestClock(Rc::new(Cell::new(0)));
    assert!(Engine::new(clock, too_short, Vec::new()).is_err());
}

#[test]
fn update_settings_rejects_invalid_values_and_keeps_the_old_ones() {
    let mut h = Harness::new(classic());
    let bad = TimerSettings {
        work_secs: 30,
        ..classic()
    };
    assert!(h.engine.update_settings(bad).is_err());
    assert_eq!(h.engine.settings(), &classic());
}

#[test]
fn update_settings_changes_a_phase_that_has_not_started() {
    let mut h = Harness::new(classic());
    let longer = TimerSettings {
        work_secs: 50 * 60,
        ..classic()
    };
    h.engine.update_settings(longer).unwrap();
    assert_eq!(h.snap().total_ms, 50 * 60 * 1000);
}

#[test]
fn update_settings_does_not_change_a_phase_in_progress() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.advance(10);
    let longer = TimerSettings {
        work_secs: 50 * 60,
        ..classic()
    };
    h.engine.update_settings(longer).unwrap();
    assert_eq!(h.snap().total_ms, 25 * 60 * 1000);
}

// ----- segments and exercises -----

#[test]
fn long_break_segments_follow_the_spec_examples() {
    assert_eq!(segment_lengths_secs(1200, 300), vec![300, 300, 300, 300]);
    assert_eq!(segment_lengths_secs(1020, 300), vec![300, 300, 420]);
    assert_eq!(segment_lengths_secs(900, 300), vec![300, 300, 300]);
    assert_eq!(segment_lengths_secs(180, 300), vec![180]);
    assert_eq!(segment_lengths_secs(600, 0), vec![600]);
}

#[test]
fn exercise_labels_include_the_optional_amount() {
    assert_eq!(exercise("Planks", Some("30 s")).label(), "Planks (30 s)");
    assert_eq!(exercise("Planks", None).label(), "Planks");
    assert_eq!(exercise("Planks", Some("  ")).label(), "Planks");
}

#[test]
fn rotation_wraps_and_ignores_empty_names() {
    let mut rotation = Rotation::new(vec![
        exercise("A", None),
        exercise("  ", None),
        exercise("B", None),
    ]);
    assert_eq!(rotation.len(), 2);
    let labels: Vec<_> = (0..5).map(|_| rotation.next_label().unwrap()).collect();
    assert_eq!(labels, ["A", "B", "A", "B", "A"]);
    assert_eq!(Rotation::default().next_label(), None);
}

#[test]
fn rotation_restarts_when_the_list_shrinks_below_its_position() {
    let mut rotation = Rotation::new(vec![
        exercise("A", None),
        exercise("B", None),
        exercise("C", None),
    ]);
    rotation.next_label();
    rotation.next_label();
    rotation.set_items(vec![exercise("X", None)]);
    assert_eq!(rotation.next_label().as_deref(), Some("X"));
}

// ----- the cycle -----

#[test]
fn a_new_engine_waits_on_a_work_phase() {
    let h = Harness::new(classic());
    let s = h.snap();
    assert_eq!(s.mode, Mode::Pomodoro);
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.phase, Some(PhaseKind::Work));
    assert_eq!(s.remaining_ms, 25 * 60 * 1000);
    assert_eq!(
        s.cycle,
        CycleInfo {
            position: 1,
            length: 4
        }
    );
    assert_eq!(s.label, None);
    assert_eq!(s.segment, None);
}

#[test]
fn nothing_ticks_until_the_user_starts() {
    let mut h = Harness::new(classic());
    assert!(h.run(10_000).is_empty());
    assert_eq!(h.snap().state, EngineState::Idle);
}

#[test]
fn work_phase_completes_and_waits_for_the_user() {
    let mut h = Harness::new(classic());
    let started = h.engine.start();
    assert_eq!(
        started,
        vec![Event::PhaseStarted {
            kind: PhaseKind::Work
        }]
    );
    assert!(h.run(1499).is_empty());
    assert_eq!(h.snap().remaining_ms, 1000);

    let events = h.run(1);
    let recorded = records(&events);
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].kind, PhaseKind::Work);
    assert_eq!(recorded[0].status, PhaseStatus::Completed);
    assert_eq!(recorded[0].planned_secs, 1500);
    assert_eq!(recorded[0].actual_secs, 1500);
    assert_eq!(recorded[0].paused_secs, 0);
    assert_eq!(recorded[0].started_unix_ms, T0);
    assert_eq!(recorded[0].ended_unix_ms, T0 + 1_500_000);
    assert_eq!(alerts(&events), vec![AlertEvent::WorkEnd]);

    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::ShortBreak));
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.remaining_ms, 300_000);
}

#[test]
fn four_work_phases_lead_to_a_long_break_and_the_cycle_restarts() {
    let mut h = Harness::new(classic());
    for round in 1..=4 {
        assert_eq!(h.snap().cycle.position, round);
        h.complete(1500);
        if round < 4 {
            assert_eq!(h.snap().phase, Some(PhaseKind::ShortBreak));
            let events = h.complete(300);
            assert_eq!(alerts(&events), vec![AlertEvent::ShortBreakEnd]);
            assert_eq!(h.snap().phase, Some(PhaseKind::Work));
        }
    }
    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::LongBreak));
    assert_eq!(s.remaining_ms, 900_000);
    assert_eq!(
        s.cycle,
        CycleInfo {
            position: 4,
            length: 4
        }
    );

    let events = h.complete(900);
    assert_eq!(alerts(&events), vec![AlertEvent::LongBreakEnd]);
    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::Work));
    assert_eq!(s.cycle.position, 1);
}

#[test]
fn auto_start_begins_the_next_phase_right_away() {
    let mut h = Harness::new(TimerSettings {
        auto_start: true,
        ..classic()
    });
    h.engine.start();
    let events = h.run(1500);
    assert!(events.contains(&Event::PhaseStarted {
        kind: PhaseKind::ShortBreak
    }));
    let s = h.snap();
    assert_eq!(s.state, EngineState::Running);
    assert_eq!(s.phase, Some(PhaseKind::ShortBreak));
    assert_eq!(s.remaining_ms, 300_000);

    let events = h.run(300);
    assert_eq!(alerts(&events), vec![AlertEvent::ShortBreakEnd]);
    assert!(events.contains(&Event::PhaseStarted {
        kind: PhaseKind::Work
    }));
    assert_eq!(h.snap().phase, Some(PhaseKind::Work));
    assert_eq!(h.snap().state, EngineState::Running);
}

#[test]
fn auto_start_after_a_long_absence_starts_the_next_phase_now() {
    let mut h = Harness::new(TimerSettings {
        auto_start: true,
        sleep_behavior: SleepBehavior::KeepRunning,
        ..classic()
    });
    h.engine.start();
    // The computer was asleep for an hour past the end of the phase.
    h.run(1500 + 3600);
    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::ShortBreak));
    assert_eq!(
        s.remaining_ms, 300_000,
        "the break starts at wake-up, not in the past"
    );
}

#[test]
fn auto_start_with_a_small_delay_keeps_the_scheduled_start() {
    let mut h = Harness::new(TimerSettings {
        auto_start: true,
        ..classic()
    });
    h.engine.start();
    h.run(1501);
    assert_eq!(h.snap().remaining_ms, 299_000);
}

// ----- pause, resume, toggle -----

#[test]
fn paused_time_does_not_count_and_is_recorded() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.run(600);
    h.engine.pause();
    assert_eq!(h.snap().state, EngineState::Paused);
    assert!(h.run(1000).is_empty());
    assert_eq!(h.snap().remaining_ms, 900_000);

    h.engine.resume();
    assert_eq!(h.snap().state, EngineState::Running);
    let events = h.run(900);
    let record = &records(&events)[0];
    assert_eq!(record.status, PhaseStatus::Completed);
    assert_eq!(record.actual_secs, 1500);
    assert_eq!(record.paused_secs, 1000);
    assert_eq!(record.ended_unix_ms - record.started_unix_ms, 2_500_000);
}

#[test]
fn pause_and_resume_do_nothing_in_the_wrong_state() {
    let mut h = Harness::new(classic());
    h.engine.pause();
    h.engine.resume();
    assert_eq!(h.snap().state, EngineState::Idle);
    h.engine.start();
    h.engine.resume();
    assert_eq!(h.snap().state, EngineState::Running);
}

#[test]
fn start_resumes_a_paused_timer() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.engine.pause();
    assert!(h.engine.start().is_empty());
    assert_eq!(h.snap().state, EngineState::Running);
}

#[test]
fn toggle_starts_pauses_and_resumes() {
    let mut h = Harness::new(classic());
    assert_eq!(h.engine.toggle().len(), 1);
    assert_eq!(h.snap().state, EngineState::Running);
    h.engine.toggle();
    assert_eq!(h.snap().state, EngineState::Paused);
    h.engine.toggle();
    assert_eq!(h.snap().state, EngineState::Running);
}

// ----- rotation position -----

#[test]
fn the_rotation_position_can_be_saved_and_restored() {
    let exercises = vec![
        Exercise {
            name: "A".into(),
            amount: None,
        },
        Exercise {
            name: "B".into(),
            amount: None,
        },
        Exercise {
            name: "C".into(),
            amount: None,
        },
    ];
    let mut h = Harness::with_exercises(classic(), exercises.clone());
    h.complete(1500);
    assert_eq!(h.snap().label.as_deref(), Some("A"));
    assert_eq!(h.engine.rotation_position(), 1);

    // A new session, as after a restart.
    let mut next = Harness::with_exercises(classic(), exercises);
    next.engine.restore_rotation(h.engine.rotation_position());
    next.complete(1500);
    assert_eq!(next.snap().label.as_deref(), Some("B"));
}

#[test]
fn a_saved_position_beyond_the_list_restarts_the_rotation() {
    let exercises = vec![
        Exercise {
            name: "A".into(),
            amount: None,
        },
        Exercise {
            name: "B".into(),
            amount: None,
        },
    ];
    let mut h = Harness::with_exercises(classic(), exercises);
    h.engine.restore_rotation(5);
    assert_eq!(h.engine.rotation_position(), 0);
}

// ----- skip -----

#[test]
fn skipping_a_work_phase_is_recorded_as_skipped_and_advances_the_cycle() {
    let mut h = Harness::new(classic());
    for _ in 0..3 {
        h.complete(1500);
        h.complete(300);
    }
    h.engine.start();
    h.advance(100);
    let events = h.engine.skip();
    let record = &records(&events)[0];
    assert_eq!(record.status, PhaseStatus::Skipped);
    assert_eq!(record.actual_secs, 100);
    assert!(alerts(&events).is_empty(), "a skip raises no alert");

    let s = h.snap();
    assert_eq!(
        s.phase,
        Some(PhaseKind::LongBreak),
        "the skipped phase is the 4th work phase of the cycle"
    );
    assert_eq!(s.state, EngineState::Idle);
}

#[test]
fn skipping_every_phase_still_reaches_the_long_break() {
    let mut h = Harness::new(classic());
    for round in 1..=4 {
        assert_eq!(h.snap().phase, Some(PhaseKind::Work));
        assert_eq!(h.snap().cycle.position, round);
        h.engine.skip();
        if round < 4 {
            assert_eq!(h.snap().phase, Some(PhaseKind::ShortBreak));
            h.engine.skip();
        }
    }
    assert_eq!(h.snap().phase, Some(PhaseKind::LongBreak));
    h.engine.skip();
    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::Work));
    assert_eq!(
        s.cycle.position, 1,
        "the cycle starts over after the long break"
    );
}

#[test]
fn skipping_a_phase_that_never_started_records_nothing() {
    let mut h = Harness::new(classic());
    let events = h.engine.skip();
    assert!(records(&events).is_empty());
    assert_eq!(h.snap().phase, Some(PhaseKind::ShortBreak));
}

#[test]
fn skip_and_start_begins_the_next_work_phase() {
    let mut h = Harness::new(classic());
    h.complete(1500);
    h.engine.start();
    h.advance(10);
    let events = h.engine.skip_and_start();
    assert_eq!(records(&events)[0].kind, PhaseKind::ShortBreak);
    assert!(events.contains(&Event::PhaseStarted {
        kind: PhaseKind::Work
    }));
    assert_eq!(h.snap().state, EngineState::Running);
    assert_eq!(h.snap().phase, Some(PhaseKind::Work));
}

#[test]
fn skipping_the_long_break_restarts_the_cycle() {
    let mut h = Harness::new(classic());
    for round in 1..=4 {
        h.complete(1500);
        if round < 4 {
            h.complete(300);
        }
    }
    assert_eq!(h.snap().phase, Some(PhaseKind::LongBreak));
    h.engine.skip();
    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::Work));
    assert_eq!(s.cycle.position, 1);
}

// ----- reset -----

#[test]
fn reset_before_start_does_nothing() {
    let mut h = Harness::new(classic());
    assert!(h.engine.reset().is_empty());
    assert_eq!(h.snap().state, EngineState::Idle);
}

#[test]
fn reset_after_start_records_an_interruption_and_restores_the_phase() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.advance(300);
    let events = h.engine.reset();
    let record = &records(&events)[0];
    assert_eq!(record.status, PhaseStatus::Interrupted);
    assert_eq!(record.actual_secs, 300);
    let s = h.snap();
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.remaining_ms, 1_500_000);
    assert_eq!(s.phase, Some(PhaseKind::Work));
}

#[test]
fn reset_keeps_the_exercise_of_the_break() {
    let mut h = Harness::with_exercises(classic(), vec![exercise("A", None), exercise("B", None)]);
    h.complete(1500);
    assert_eq!(h.snap().label.as_deref(), Some("A"));
    h.engine.start();
    h.advance(10);
    h.engine.reset();
    assert_eq!(h.snap().label.as_deref(), Some("A"));
}

// ----- extend -----

#[test]
fn extending_a_break_adds_time_and_is_recorded() {
    let mut h = Harness::new(classic());
    h.complete(1500);
    h.engine.start();
    h.advance(100);
    h.engine.extend(60);
    let s = h.snap();
    assert_eq!(s.total_ms, 360_000);
    assert_eq!(s.remaining_ms, 260_000);

    let events = h.run(260);
    let record = &records(&events)[0];
    assert_eq!(record.planned_secs, 300);
    assert_eq!(record.extended_secs, 60);
    assert_eq!(record.actual_secs, 360);
}

#[test]
fn extend_does_nothing_for_work_phases_or_unstarted_breaks() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.engine.extend(60);
    assert_eq!(h.snap().total_ms, 1_500_000);

    h.run(1500);
    h.engine.extend(60);
    assert_eq!(h.snap().total_ms, 300_000, "the break has not started");
}

// ----- exercises and long break segments -----

#[test]
fn short_breaks_rotate_through_the_exercise_list_in_order() {
    let mut h = Harness::with_exercises(
        TimerSettings {
            sessions_before_long: 12,
            ..classic()
        },
        vec![
            exercise("Planks", Some("30 s")),
            exercise("Sit-ups", Some("15 reps")),
            exercise("Stretch", None),
        ],
    );
    let mut seen = Vec::new();
    for _ in 0..4 {
        let events = h.complete(1500);
        seen.push(h.snap().label.expect("a break label"));
        assert_eq!(
            alert_label(&events),
            h.snap().label,
            "the alert carries the label"
        );
        h.engine.skip();
    }
    assert_eq!(
        seen,
        [
            "Planks (30 s)",
            "Sit-ups (15 reps)",
            "Stretch",
            "Planks (30 s)"
        ]
    );
}

#[test]
fn breaks_have_no_label_without_exercises() {
    let mut h = Harness::new(classic());
    let events = h.complete(1500);
    assert_eq!(h.snap().label, None);
    assert_eq!(alert_label(&events), None);
}

#[test]
fn a_long_break_walks_through_the_list_one_segment_at_a_time() {
    let mut h = Harness::with_exercises(
        TimerSettings {
            long_break_secs: 1200,
            sessions_before_long: 2,
            ..classic()
        },
        ["A", "B", "C", "D", "E"]
            .iter()
            .map(|name| exercise(name, None))
            .collect(),
    );
    h.complete(1500);
    h.complete(300); // short break shows "A"
    h.complete(1500);

    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::LongBreak));
    assert_eq!(s.label.as_deref(), Some("B"));
    assert_eq!(
        s.segment,
        Some(SegmentInfo {
            index: 0,
            count: 4,
            total_ms: 300_000,
            remaining_ms: 300_000
        })
    );

    h.engine.start();
    assert!(h.run(299).is_empty());
    let events = h.run(1);
    assert_eq!(
        events,
        vec![Event::SegmentChanged {
            index: 1,
            count: 4,
            label: Some("C".into())
        }]
    );
    assert_eq!(h.snap().label.as_deref(), Some("C"));

    assert_eq!(
        h.run(300),
        vec![Event::SegmentChanged {
            index: 2,
            count: 4,
            label: Some("D".into())
        }]
    );
    assert_eq!(
        h.run(300),
        vec![Event::SegmentChanged {
            index: 3,
            count: 4,
            label: Some("E".into())
        }]
    );
    assert!(h.run(299).is_empty());

    let events = h.run(1);
    let record = &records(&events)[0];
    assert_eq!(record.kind, PhaseKind::LongBreak);
    assert_eq!(record.labels, ["B", "C", "D", "E"]);
    assert_eq!(alerts(&events), vec![AlertEvent::LongBreakEnd]);

    // The rotation carries on after the long break.
    h.complete(1500);
    assert_eq!(h.snap().phase, Some(PhaseKind::ShortBreak));
    assert_eq!(h.snap().label.as_deref(), Some("A"));
}

#[test]
fn a_long_break_remainder_goes_to_the_last_segment() {
    let mut h = Harness::new(TimerSettings {
        long_break_secs: 1020,
        sessions_before_long: 2,
        ..classic()
    });
    h.complete(1500);
    h.complete(300);
    h.complete(1500);
    h.engine.start();
    h.advance(650);
    let segment = h.snap().segment.expect("segment info during a long break");
    assert_eq!(segment.index, 2);
    assert_eq!(segment.count, 3);
    assert_eq!(segment.total_ms, 420_000);
    assert_eq!(segment.remaining_ms, 370_000);
}

#[test]
fn extending_a_long_break_lengthens_the_last_segment() {
    let mut h = Harness::new(TimerSettings {
        long_break_secs: 1200,
        sessions_before_long: 2,
        ..classic()
    });
    h.complete(1500);
    h.complete(300);
    h.complete(1500);
    h.engine.start();
    h.advance(10);
    h.engine.extend(300);
    h.advance(990);
    let segment = h.snap().segment.unwrap();
    assert_eq!(segment.index, 3);
    assert_eq!(segment.total_ms, 600_000);
    assert_eq!(segment.remaining_ms, 500_000);
}

// ----- sleep and lock -----

#[test]
fn auto_pause_holds_the_timer_until_the_user_resumes() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.run(100);
    h.engine.on_system_suspend();
    assert_eq!(h.snap().state, EngineState::AutoPaused);

    assert!(h.run(3600).is_empty());
    assert_eq!(h.snap().remaining_ms, 1_400_000);
    assert_eq!(h.snap().state, EngineState::AutoPaused);

    h.engine.resume();
    assert_eq!(h.snap().state, EngineState::Running);
    let events = h.run(1400);
    assert_eq!(records(&events)[0].paused_secs, 3600);
}

#[test]
fn a_suspend_noticed_on_wake_pauses_as_of_when_it_began() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.run(100);
    let began = h.engine.now_ms();
    h.advance(3600);
    // Nobody ticked during the sleep, so the timer still thinks it runs.
    h.engine.on_system_suspend_at(began);
    assert_eq!(h.snap().state, EngineState::AutoPaused);
    assert_eq!(h.snap().remaining_ms, 1_400_000);

    h.engine.resume();
    let events = h.run(1400);
    assert_eq!(records(&events)[0].paused_secs, 3600);
}

#[test]
fn suspend_does_nothing_to_a_timer_that_is_not_running() {
    let mut h = Harness::new(classic());
    h.engine.on_system_suspend();
    assert_eq!(h.snap().state, EngineState::Idle);
    h.engine.start();
    h.engine.pause();
    h.engine.on_system_suspend();
    assert_eq!(
        h.snap().state,
        EngineState::Paused,
        "a manual pause stays manual"
    );
}

#[test]
fn keep_running_lets_a_phase_end_while_the_computer_sleeps() {
    let mut h = Harness::new(TimerSettings {
        sleep_behavior: SleepBehavior::KeepRunning,
        ..classic()
    });
    h.engine.start();
    h.run(100);
    h.engine.on_system_suspend();
    assert_eq!(h.snap().state, EngineState::Running);
    let events = h.run(2000);
    assert_eq!(alerts(&events), vec![AlertEvent::WorkEnd]);
}

// ----- modes -----

#[test]
fn leaving_pomodoro_mode_resets_it_and_records_the_interruption() {
    let mut h = Harness::new(classic());
    h.complete(1500);
    h.complete(300);
    h.engine.start();
    h.advance(100);
    assert_eq!(h.snap().cycle.position, 2);

    let events = h.engine.set_mode(Mode::Countdown);
    let record = &records(&events)[0];
    assert_eq!(record.status, PhaseStatus::Interrupted);
    assert_eq!(record.kind, PhaseKind::Work);

    let s = h.snap();
    assert_eq!(s.mode, Mode::Countdown);
    assert_eq!(s.phase, None);
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.remaining_ms, 600_000);

    h.engine.set_mode(Mode::Pomodoro);
    let s = h.snap();
    assert_eq!(s.phase, Some(PhaseKind::Work));
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.remaining_ms, 1_500_000);
    assert_eq!(s.cycle.position, 1, "the cycle restarts");
}

#[test]
fn leaving_countdown_mode_resets_the_countdown() {
    let mut h = Harness::new(classic());
    h.engine.set_mode(Mode::Countdown);
    h.engine.start();
    h.advance(100);
    assert_eq!(h.snap().remaining_ms, 500_000);
    let events = h.engine.set_mode(Mode::Pomodoro);
    assert!(events.is_empty(), "countdowns are not recorded");
    h.engine.set_mode(Mode::Countdown);
    let s = h.snap();
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.remaining_ms, 600_000);
}

#[test]
fn switching_to_the_same_mode_changes_nothing() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.advance(100);
    assert!(h.engine.set_mode(Mode::Pomodoro).is_empty());
    assert_eq!(h.snap().state, EngineState::Running);
}

#[test]
fn only_one_mode_runs_at_a_time() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.engine.set_mode(Mode::Countdown);
    // The Pomodoro phase that was running is gone: letting its time pass
    // raises no alert and nothing runs in the background.
    assert!(h.run(1500).is_empty());
    assert_eq!(h.snap().state, EngineState::Idle);
}

// ----- countdown -----

#[test]
fn countdown_alerts_once_at_zero_and_is_not_recorded() {
    let mut h = Harness::new(classic());
    h.engine.set_mode(Mode::Countdown);
    h.engine.set_countdown(90).unwrap();
    assert_eq!(h.snap().remaining_ms, 90_000);

    assert!(
        h.engine.start().is_empty(),
        "no phase events in countdown mode"
    );
    assert!(h.run(89).is_empty());
    let events = h.run(1);
    assert_eq!(
        events,
        vec![Event::Alert {
            event: AlertEvent::CountdownEnd,
            label: None
        }]
    );
    assert!(records(&events).is_empty());

    let s = h.snap();
    assert_eq!(s.state, EngineState::Idle);
    assert_eq!(s.remaining_ms, 90_000);

    // Restart
    h.engine.start();
    assert_eq!(h.snap().state, EngineState::Running);
}

#[test]
fn countdown_pauses_and_resets() {
    let mut h = Harness::new(classic());
    h.engine.set_mode(Mode::Countdown);
    h.engine.start();
    h.run(100);
    h.engine.pause();
    h.run(1000);
    assert_eq!(h.snap().remaining_ms, 500_000);
    h.engine.resume();
    h.engine.reset();
    assert_eq!(h.snap().remaining_ms, 600_000);
    assert_eq!(h.snap().state, EngineState::Idle);
}

#[test]
fn countdown_ignores_skip_and_extend() {
    let mut h = Harness::new(classic());
    h.engine.set_mode(Mode::Countdown);
    h.engine.start();
    assert!(h.engine.skip().is_empty());
    h.engine.extend(60);
    let s = h.snap();
    assert_eq!(s.total_ms, 600_000);
    assert_eq!(s.state, EngineState::Running);
}

#[test]
fn countdown_duration_is_validated() {
    let mut h = Harness::new(classic());
    assert!(h.engine.set_countdown(0).is_err());
    assert!(h.engine.set_countdown(24 * 60 * 60).is_err());
    assert!(h.engine.set_countdown(24 * 60 * 60 - 1).is_ok());
    assert!(h.engine.set_countdown(1).is_ok());
}

// ----- shutdown -----

#[test]
fn shutdown_records_the_phase_in_progress_as_interrupted() {
    let mut h = Harness::new(classic());
    h.engine.start();
    h.advance(100);
    let events = h.engine.shutdown();
    let record = &records(&events)[0];
    assert_eq!(record.status, PhaseStatus::Interrupted);
    assert_eq!(record.actual_secs, 100);
    assert!(h.engine.shutdown().is_empty(), "nothing left to record");
}

// ----- frontend contract -----

#[test]
fn events_and_snapshots_serialize_with_snake_case_names() {
    let mut h = Harness::with_exercises(classic(), vec![exercise("Planks", Some("30 s"))]);
    h.engine.start();
    let events = h.run(1500);

    let alert = serde_json::to_value(&events[1]).unwrap();
    assert_eq!(
        alert,
        serde_json::json!({ "type": "alert", "event": "work_end", "label": "Planks (30 s)" })
    );
    let record = serde_json::to_value(&events[0]).unwrap();
    assert_eq!(record["type"], "phase_recorded");
    assert_eq!(record["status"], "completed");
    assert_eq!(record["kind"], "work");

    let snapshot = serde_json::to_value(h.snap()).unwrap();
    assert_eq!(snapshot["mode"], "pomodoro");
    assert_eq!(snapshot["state"], "idle");
    assert_eq!(snapshot["phase"], "short_break");
    assert_eq!(snapshot["remaining_ms"], 300_000);
    assert_eq!(snapshot["cycle"]["length"], 4);

    h.engine.on_system_suspend();
    h.engine.start();
    h.engine.on_system_suspend();
    let snapshot = serde_json::to_value(h.snap()).unwrap();
    assert_eq!(snapshot["state"], "auto_paused");
}
