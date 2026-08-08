//! Aircraft state: the numbers that exist between ticks and evolve as the
//! sim runs.
//!
//! The pitch/bank -> vertical-speed/heading coupling below is a
//! deliberately simplified placeholder, not real performance modeling --
//! that belongs to `physics/` once it exists. What this issue actually
//! establishes is the state itself, first-order response toward control
//! targets, and integration wired into the tick loop in a way later
//! issues can build real aerodynamics on top of without restructuring.

use crate::aircraft::controls::ControlInputs;
use crate::aircraft::engines::EngineState;

/// Pitch, in degrees, that holds level flight at cruise trim. Used as the
/// zero point for the pitch -> vertical-speed coupling: at exactly this
/// pitch, vertical speed is zero.
const REFERENCE_LEVEL_PITCH_DEG: f64 = ControlInputs::CRUISE_TRIM.pitch_target_deg;

/// How many feet per minute of climb/descent one degree of pitch away
/// from level produces. Not derived from real aircraft performance data
/// -- picked to give a believable-feeling response.
const FPM_PER_PITCH_DEGREE: f64 = 300.0;

/// How fast pitch/bank move toward their control targets, in degrees per
/// second. A first-order lag rather than an instant snap, same idea as
/// the engine's spool rate.
const PITCH_RATE_PER_SECOND: f64 = 3.0;
const BANK_RATE_PER_SECOND: f64 = 5.0;

/// Degrees of heading change per second, per degree of bank held. Rough
/// standard-rate-turn feel, not a real turn-radius/bank-angle formula.
const HEADING_DEG_PER_SECOND_PER_BANK_DEG: f64 = 0.15;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AircraftState {
    pub altitude_ft: f64,
    pub vertical_speed_fpm: f64,
    pub heading_deg: f64,
    pub pitch_deg: f64,
    pub bank_deg: f64,
    /// Not modeled yet -- thrust/drag -> airspeed coupling belongs to
    /// physics/performance, a later issue. Held here so the PFD has a
    /// value to display; it will not move on its own until that lands.
    pub indicated_airspeed_kt: f64,
    pub engine: EngineState,
}

impl AircraftState {
    /// Stable, trimmed level cruise -- matches `ControlInputs::CRUISE_TRIM`
    /// so a fresh sim starts in equilibrium instead of immediately
    /// drifting toward the default targets.
    pub fn cruise() -> Self {
        Self {
            altitude_ft: 3_000.0,
            vertical_speed_fpm: 0.0,
            heading_deg: 90.0,
            pitch_deg: ControlInputs::CRUISE_TRIM.pitch_target_deg,
            bank_deg: ControlInputs::CRUISE_TRIM.bank_target_deg,
            indicated_airspeed_kt: 250.0,
            engine: EngineState::new(),
        }
    }

    /// Advance the state by `dt_seconds` given the current control
    /// targets. Called once per tick from `Simulation::tick`.
    pub fn integrate(&mut self, controls: &ControlInputs, dt_seconds: f64) {
        self.pitch_deg =
            approach(self.pitch_deg, controls.pitch_target_deg, PITCH_RATE_PER_SECOND, dt_seconds);
        self.bank_deg =
            approach(self.bank_deg, controls.bank_target_deg, BANK_RATE_PER_SECOND, dt_seconds);
        self.engine.integrate(controls.thrust_target_percent, dt_seconds);

        self.vertical_speed_fpm =
            (self.pitch_deg - REFERENCE_LEVEL_PITCH_DEG) * FPM_PER_PITCH_DEGREE;
        self.altitude_ft += self.vertical_speed_fpm / 60.0 * dt_seconds;
        // There's no ground/terrain model yet (that's world::airport +
        // a real terrain-awareness issue, not scoped yet) -- but letting
        // altitude go negative is clearly wrong in the meantime, so it's
        // floored at zero rather than left unbounded.
        self.altitude_ft = self.altitude_ft.max(0.0);

        let heading_rate = self.bank_deg * HEADING_DEG_PER_SECOND_PER_BANK_DEG;
        self.heading_deg = wrap_heading(self.heading_deg + heading_rate * dt_seconds);
    }
}

impl Default for AircraftState {
    fn default() -> Self {
        Self::cruise()
    }
}

/// Move `current` toward `target` at `rate_per_second`, without
/// overshooting past `target` in one step.
fn approach(current: f64, target: f64, rate_per_second: f64, dt_seconds: f64) -> f64 {
    let max_step = rate_per_second * dt_seconds;
    let gap = target - current;
    if gap.abs() <= max_step {
        target
    } else {
        current + max_step * gap.signum()
    }
}

fn wrap_heading(heading_deg: f64) -> f64 {
    heading_deg.rem_euclid(360.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cruise_state_is_in_equilibrium_under_default_controls() {
        let mut state = AircraftState::cruise();
        let controls = ControlInputs::default();
        let initial_altitude = state.altitude_ft;

        for _ in 0..100 {
            state.integrate(&controls, 0.1);
        }

        assert_eq!(state.vertical_speed_fpm, 0.0);
        assert_eq!(state.altitude_ft, initial_altitude);
    }

    #[test]
    fn pitching_up_from_trim_eventually_produces_climb() {
        let mut state = AircraftState::cruise();
        let controls = ControlInputs { pitch_target_deg: 7.5, ..ControlInputs::default() };

        for _ in 0..50 {
            state.integrate(&controls, 0.1);
        }

        assert!(state.vertical_speed_fpm > 0.0);
        assert!(state.altitude_ft > 3_000.0);
    }

    #[test]
    fn banking_changes_heading_over_time() {
        let mut state = AircraftState::cruise();
        let controls = ControlInputs { bank_target_deg: 20.0, ..ControlInputs::default() };
        let initial_heading = state.heading_deg;

        for _ in 0..100 {
            state.integrate(&controls, 0.1);
        }

        assert_ne!(state.heading_deg, initial_heading);
    }

    #[test]
    fn altitude_does_not_go_negative_on_a_sustained_descent() {
        let mut state = AircraftState { altitude_ft: 100.0, ..AircraftState::cruise() };
        let controls = ControlInputs { pitch_target_deg: -10.0, ..ControlInputs::default() };

        for _ in 0..600 {
            state.integrate(&controls, 0.1);
        }

        assert_eq!(state.altitude_ft, 0.0);
    }

    #[test]
    fn heading_wraps_past_360() {
        assert_eq!(wrap_heading(370.0), 10.0);
        assert_eq!(wrap_heading(-10.0), 350.0);
    }

    #[test]
    fn approach_does_not_overshoot_target_in_one_large_step() {
        assert_eq!(approach(0.0, 10.0, 3.0, 100.0), 10.0);
    }
}
