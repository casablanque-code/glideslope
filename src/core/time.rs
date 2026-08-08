//! Simulation time.
//!
//! The sim never reads the wall clock directly outside of the top-level
//! loop. Everything else operates on [`SimTime`] / tick counts so that a
//! recorded replay is reproducible regardless of how fast real time
//! actually passed while it was recorded.

use std::time::Duration;

/// How many simulation ticks happen per second of sim time.
///
/// 10 Hz is coarse enough to be cheap and fine enough that "a few seconds"
/// of in-fiction time (reading an instrument, keying the radio) is several
/// ticks, which is what the workload-management loop depends on.
pub const TICKS_PER_SECOND: u32 = 10;

pub const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICKS_PER_SECOND as u64);

/// A point in simulation time, expressed as an elapsed tick count.
///
/// Deliberately not wall-clock time: two runs with the same event log
/// produce the same `SimTime` sequence even if real time elapsed
/// differently (paused, slow machine, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct SimTime {
    ticks: u64,
}

impl SimTime {
    pub const ZERO: SimTime = SimTime { ticks: 0 };

    // Used by tests and by callers constructing a SimTime from a stored
    // tick count (e.g. replay, issue #TBD) — not yet called from the
    // running sim itself, hence `allow(dead_code)` rather than deleting it.
    #[allow(dead_code)]
    pub fn from_ticks(ticks: u64) -> Self {
        Self { ticks }
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    #[allow(dead_code)] // reserved for replay/debug formatting, unused so far
    pub fn as_duration(&self) -> Duration {
        TICK_DURATION * self.ticks as u32
    }

    pub fn advance(&mut self, ticks: u64) {
        self.ticks += ticks;
    }
}

/// Owns the simulation's current time and advances it one whole tick at a
/// time. This is the single source of truth for "what tick are we on" —
/// subsystems read it, nothing but the sim loop advances it.
#[derive(Debug, Default)]
pub struct SimClock {
    now: SimTime,
}

impl SimClock {
    pub fn new() -> Self {
        Self { now: SimTime::ZERO }
    }

    pub fn now(&self) -> SimTime {
        self.now
    }

    pub fn tick_count(&self) -> u64 {
        self.now.ticks()
    }

    /// Advance the clock by exactly one tick. Called once per tick by the
    /// sim loop — never by subsystems.
    pub fn tick(&mut self) {
        self.now.advance(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_starts_at_zero_and_ticks_by_one() {
        let mut clock = SimClock::new();
        assert_eq!(clock.tick_count(), 0);
        clock.tick();
        clock.tick();
        assert_eq!(clock.tick_count(), 2);
    }

    #[test]
    fn advance_accumulates_ticks() {
        let mut t = SimTime::ZERO;
        t.advance(5);
        t.advance(3);
        assert_eq!(t.ticks(), 8);
    }

    #[test]
    fn duration_matches_tick_rate() {
        let t = SimTime::from_ticks(TICKS_PER_SECOND as u64);
        assert_eq!(t.as_duration(), Duration::from_secs(1));
    }
}
