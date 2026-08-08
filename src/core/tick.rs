//! Fixed-timestep scheduling.
//!
//! Subsystems (aircraft, crew, weather, ...) must all advance in lockstep,
//! one sim tick at a time, regardless of how the outer loop is driven (a
//! `sleep`-based loop today, a real UI event loop later, or a replay
//! reading ticks from a log). [`FixedTimestep`] is the piece that turns
//! "some amount of wall-clock time passed" into "advance the sim N whole
//! ticks", using an accumulator so time is never lost or double-counted.

use crate::core::time::TICK_DURATION;
use std::time::Duration;

/// Accumulator-based fixed timestep driver.
///
/// Feed it how much wall-clock time elapsed via [`FixedTimestep::advance`],
/// then drain whole ticks with [`FixedTimestep::step`]. This is the
/// standard "fix your timestep" pattern: it decouples simulation
/// determinism from frame/poll rate.
#[derive(Debug, Default)]
pub struct FixedTimestep {
    accumulator: Duration,
}

impl FixedTimestep {
    pub fn new() -> Self {
        Self {
            accumulator: Duration::ZERO,
        }
    }

    /// Record that `elapsed` wall-clock time has passed since the last call.
    pub fn advance(&mut self, elapsed: Duration) {
        self.accumulator += elapsed;
    }

    /// Consume one pending tick's worth of accumulated time, if there is
    /// one. Call this in a loop until it returns `false` to drain all
    /// ticks owed for the current frame — a single slow frame (e.g. after
    /// a debugger pause) should produce multiple sim ticks, not one giant
    /// one.
    pub fn step(&mut self) -> bool {
        if self.accumulator >= TICK_DURATION {
            self.accumulator -= TICK_DURATION;
            true
        } else {
            false
        }
    }

    /// Fraction (0.0..1.0) of the way into the next tick. Reserved for
    /// interpolated rendering later; unused until the UI issue lands.
    #[allow(dead_code)]
    pub fn alpha(&self) -> f64 {
        self.accumulator.as_secs_f64() / TICK_DURATION.as_secs_f64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_ticks_before_one_tick_duration_elapses() {
        let mut fts = FixedTimestep::new();
        fts.advance(TICK_DURATION / 2);
        assert!(!fts.step());
    }

    #[test]
    fn one_tick_duration_yields_exactly_one_tick() {
        let mut fts = FixedTimestep::new();
        fts.advance(TICK_DURATION);
        assert!(fts.step());
        assert!(!fts.step());
    }

    #[test]
    fn a_long_stall_yields_multiple_ticks_not_one() {
        let mut fts = FixedTimestep::new();
        fts.advance(TICK_DURATION * 5);
        let mut count = 0;
        while fts.step() {
            count += 1;
        }
        assert_eq!(count, 5);
    }
}
