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
    /// The pitch, in degrees, that gives zero vertical speed for
    /// whatever trim this state started at -- not a real aircraft's
    /// AoA/speed relationship (which we don't model), just the pitch
    /// value each constructor is already defined to be in equilibrium
    /// at. Was previously a single hardcoded module constant tied to
    /// cruise trim specifically; that was wrong for any other starting
    /// trim (a parked, level aircraft at 0deg pitch was computed as
    /// descending, because 2.5deg -- cruise's trim -- was being treated
    /// as universal "level"). Per-instance until there's a real
    /// AoA/speed model to replace this with.
    trim_reference_pitch_deg: f64,
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
            trim_reference_pitch_deg: ControlInputs::CRUISE_TRIM.pitch_target_deg,
        }
    }

    /// On the ground at a gate: engines off, stationary, wings level,
    /// nose level -- in equilibrium with `ControlInputs::GATE_TRIM`, the
    /// same way `cruise()` matches `CRUISE_TRIM`. Takes plain values
    /// (elevation, heading) rather than a `world::airport::Airport`
    /// directly, so `aircraft::state` doesn't need to depend on
    /// `world::airport` -- the caller (`Simulation`) reads those fields
    /// off whatever airport it's using.
    ///
    /// Note this doesn't add any ground-contact enforcement: nothing
    /// stops `integrate()` from "climbing" a stationary, engines-off
    /// aircraft if pitch is commanded up, because the current pitch ->
    /// vertical-speed coupling was never tied to thrust or airspeed in
    /// the first place (see the module doc). That's the same documented
    /// simplification as always, just more visible now that idle is a
    /// starting state instead of only a cruise trim value.
    pub fn at_gate(elevation_ft: f64, heading_deg: f64) -> Self {
        Self {
            altitude_ft: elevation_ft,
            vertical_speed_fpm: 0.0,
            heading_deg,
            pitch_deg: ControlInputs::GATE_TRIM.pitch_target_deg,
            bank_deg: ControlInputs::GATE_TRIM.bank_target_deg,
            indicated_airspeed_kt: 0.0,
            engine: EngineState::new(),
            trim_reference_pitch_deg: ControlInputs::GATE_TRIM.pitch_target_deg,
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
            (self.pitch_deg - self.trim_reference_pitch_deg) * FPM_PER_PITCH_DEGREE;
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
    fn gate_state_is_in_equilibrium_under_gate_trim() {
        let mut state = AircraftState::at_gate(620.0, 90.0);
        let controls = ControlInputs::GATE_TRIM;

        for _ in 0..100 {
            state.integrate(&controls, 0.1);
        }

        assert_eq!(state.altitude_ft, 620.0);
        assert_eq!(state.indicated_airspeed_kt, 0.0);
        assert_eq!(state.engine.n1_percent, 0.0);
    }

    #[test]
    fn pitching_up_from_the_gate_still_produces_a_climb() {
        // Regression test for the bug this fix addresses: the zero-VS
        // reference pitch used to be hardcoded to cruise trim (2.5deg),
        // so a gate-started aircraft sitting level at 0deg was wrongly
        // computed as already descending. It must start in equilibrium
        // (covered above) *and* still respond to a pitch command.
        let mut state = AircraftState::at_gate(620.0, 90.0);
        let controls = ControlInputs { pitch_target_deg: 7.5, ..ControlInputs::GATE_TRIM };

        for _ in 0..50 {
            state.integrate(&controls, 0.1);
        }

        assert!(state.vertical_speed_fpm > 0.0);
        assert!(state.altitude_ft > 620.0);
    }

    #[test]
    fn at_gate_uses_the_given_elevation_and_heading() {
        let state = AircraftState::at_gate(1_234.0, 270.0);
        assert_eq!(state.altitude_ft, 1_234.0);
        assert_eq!(state.heading_deg, 270.0);
        assert_eq!(state.indicated_airspeed_kt, 0.0);
    }

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
