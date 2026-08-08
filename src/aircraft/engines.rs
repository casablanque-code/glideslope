//! Minimal single-engine model.
//!
//! Just enough to give the ENGINE panel a real number: N1 chases its
//! commanded target with a first-order lag instead of snapping instantly,
//! which is closer to how a turbine actually spools. No fuel flow, EGT,
//! N2, or failure modes yet -- those are separate concerns for later
//! issues (failures.rs doesn't exist until #10).

/// How much of the gap to the target N1 is closed per second. Larger
/// values spool faster. Chosen to feel responsive over a few seconds
/// without being instant -- not sourced from a real engine's spool
/// characteristics.
const SPOOL_RATE_PER_SECOND: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EngineState {
    pub n1_percent: f64,
}

impl EngineState {
    pub fn new() -> Self {
        Self { n1_percent: 0.0 }
    }

    /// Move `n1_percent` a fraction of the way toward `target_percent`,
    /// scaled by how much time passed. Calling this every tick with the
    /// same target converges smoothly rather than jumping.
    pub fn integrate(&mut self, target_percent: f64, dt_seconds: f64) {
        let gap = target_percent - self.n1_percent;
        self.n1_percent += gap * SPOOL_RATE_PER_SECOND * dt_seconds;
    }
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n1_climbs_toward_target_but_does_not_overshoot_in_one_step() {
        let mut engine = EngineState::new();
        engine.integrate(85.0, 1.0);
        assert!(engine.n1_percent > 0.0);
        assert!(engine.n1_percent < 85.0);
    }

    #[test]
    fn n1_converges_to_target_over_many_ticks() {
        let mut engine = EngineState::new();
        for _ in 0..500 {
            engine.integrate(85.0, 0.1);
        }
        assert!((engine.n1_percent - 85.0).abs() < 0.1);
    }

    #[test]
    fn zero_elapsed_time_does_not_change_n1() {
        let mut engine = EngineState { n1_percent: 40.0 };
        engine.integrate(85.0, 0.0);
        assert_eq!(engine.n1_percent, 40.0);
    }
}
