use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Time source for the engine. Injectable so cycle logic can be tested
/// without waiting.
pub trait Clock {
    /// Milliseconds on a clock that never goes backwards. Only differences
    /// between two readings matter.
    fn now_ms(&self) -> u64;

    /// Wall-clock milliseconds since the Unix epoch, used for history
    /// timestamps.
    fn unix_ms(&self) -> u64;
}

/// The real clock.
///
/// The monotonic reading is derived from wall time and clamped so it never
/// goes backwards. Operating system monotonic clocks stop while the computer
/// sleeps on some platforms, which would break the "keep running" sleep
/// behavior: a phase that ends while the lid is closed must end on time.
#[derive(Debug, Default)]
pub struct SystemClock {
    last_ms: AtomicU64,
}

impl SystemClock {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        let reading = self.unix_ms();
        let previous = self.last_ms.fetch_max(reading, Ordering::Relaxed);
        previous.max(reading)
    }

    fn unix_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}
