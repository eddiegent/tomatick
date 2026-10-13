//! Noticing that the computer slept (SPEC.md section 5.4).
//!
//! Power sources sit behind the [`PowerListener`] trait, so a platform
//! notification (a session lock on Windows, logind on Linux, the workspace
//! notifications on macOS) can be added next to the portable one without
//! touching the host. This module does not depend on Tauri.

/// A gap between two polls longer than this means the computer slept (or the
/// app was frozen for that long).
pub const SLEEP_GAP_MS: u64 = 10_000;

/// Something the power source noticed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerEvent {
    /// The computer was suspended. `since_ms` is the last time the app was
    /// known to be awake, on the engine clock.
    Suspended { since_ms: u64 },
}

/// A source of power events. Polled by the host on every tick with the
/// current engine time.
pub trait PowerListener: Send {
    fn poll(&mut self, now_ms: u64) -> Vec<PowerEvent>;
}

/// Works everywhere: a computer that sleeps stops the ticker, so the next
/// poll comes much later than expected. It cannot tell a sleep from a
/// screen lock, which is why it only reports sleeps.
#[derive(Debug)]
pub struct GapDetector {
    threshold_ms: u64,
    last_ms: Option<u64>,
}

impl GapDetector {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            threshold_ms,
            last_ms: None,
        }
    }
}

impl Default for GapDetector {
    fn default() -> Self {
        Self::new(SLEEP_GAP_MS)
    }
}

impl PowerListener for GapDetector {
    fn poll(&mut self, now_ms: u64) -> Vec<PowerEvent> {
        let previous = self.last_ms.replace(now_ms);
        match previous {
            Some(last) if now_ms.saturating_sub(last) > self.threshold_ms => {
                vec![PowerEvent::Suspended { since_ms: last }]
            }
            _ => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regular_polls_report_nothing() {
        let mut detector = GapDetector::default();
        for step in 0..100 {
            assert!(detector.poll(step * 250).is_empty());
        }
    }

    #[test]
    fn a_long_gap_reports_when_the_app_was_last_awake() {
        let mut detector = GapDetector::default();
        assert!(detector.poll(1_000).is_empty());
        assert!(detector.poll(1_250).is_empty());
        assert_eq!(
            detector.poll(3_600_000),
            vec![PowerEvent::Suspended { since_ms: 1_250 }]
        );
        assert!(detector.poll(3_600_250).is_empty(), "reported only once");
    }

    #[test]
    fn a_gap_just_under_the_threshold_is_not_a_sleep() {
        let mut detector = GapDetector::new(10_000);
        detector.poll(0);
        assert!(detector.poll(10_000).is_empty());
        assert_eq!(detector.poll(20_001).len(), 1);
    }
}
