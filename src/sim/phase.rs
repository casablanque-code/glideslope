//! Flight phase state machine.
//!
//! See DESIGN.md for the full design discussion (why taxi is
//! abstracted, why gating is soft, why holding isn't modeled yet). This
//! is the implementation of that decision: every transition here
//! follows from a real, observable condition -- an ATC clearance grant,
//! an altitude crossing, an engine spool-up, or an explicit player
//! command -- never a timer.
//!
//! Holding ("Hold due traffic") is deliberately not modeled -- nothing
//! generates a real reason to hold yet (no traffic system), and per
//! DESIGN.md, holding is meant to be a modifier on the current phase,
//! not a phase of its own; building that modifier ahead of anything
//! that could set it would be inert scaffolding.

use crate::atc::constraints::ClearanceType;
use crate::atc::controller::Controller;

/// The MVP's single cruise altitude. Not derived from flight planning
/// (no route/altitude-selection system exists) -- just the one level
/// Climb -> Cruise transitions at.
pub const CRUISE_ALTITUDE_FT: f64 = 3_000.0;

/// How far above the runway's elevation counts as "airborne" (Takeoff ->
/// Climb) or as "touched down" when descending through it (Landing ->
/// TaxiToGate). Not a gear-squat-switch simulation -- a margin around an
/// altitude reading that could plausibly be noisy if it were real.
const GROUND_MARGIN_FT: f64 = 10.0;

/// N1 percent above which the aircraft is considered to have started
/// its takeoff roll (TakeoffHold -> Takeoff). Not a V1/rotate-speed
/// check -- there's no ground-roll acceleration model -- just "the
/// player has actually advanced the throttles," a real and observable
/// condition rather than a scripted delay.
const TAKEOFF_ROLL_N1_PERCENT: f64 = 50.0;

/// Altitude above the runway's elevation below which Approach ->
/// Landing may occur, alongside a landing clearance. A physical
/// precondition (an aircraft at cruise altitude cannot be "landing"),
/// not an artificial game rule -- see DESIGN.md's hard-vs-soft-gate
/// note. Not derived from real stabilized-approach criteria (needs ILS,
/// #9) -- a placeholder proxy until that exists.
const LANDING_ALTITUDE_MARGIN_FT: f64 = 500.0;

/// How much altitude must be gained above where a go-around began before
/// it's considered complete (GoAround -> Climb). Using a gain rather
/// than an absolute altitude means it works the same regardless of how
/// low the go-around started.
const GO_AROUND_CLIMB_GAIN_FT: f64 = 500.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightPhase {
    Ground,
    Taxi,
    TakeoffHold,
    Takeoff,
    Climb,
    Cruise,
    Descent,
    Approach,
    Landing,
    GoAround,
    TaxiToGate,
    Shutdown,
}

impl FlightPhase {
    pub fn name(&self) -> &'static str {
        match self {
            FlightPhase::Ground => "GROUND",
            FlightPhase::Taxi => "TAXI",
            FlightPhase::TakeoffHold => "TAKEOFF HOLD",
            FlightPhase::Takeoff => "TAKEOFF",
            FlightPhase::Climb => "CLIMB",
            FlightPhase::Cruise => "CRUISE",
            FlightPhase::Descent => "DESCENT",
            FlightPhase::Approach => "APPROACH",
            FlightPhase::Landing => "LANDING",
            FlightPhase::GoAround => "GO-AROUND",
            FlightPhase::TaxiToGate => "TAXI TO GATE",
            FlightPhase::Shutdown => "SHUTDOWN",
        }
    }
}

/// Everything the automatic-transition check needs to read. A plain
/// struct rather than passing four loose arguments, and rather than
/// giving `PhaseTracker` a reference to `Simulation` itself -- this way
/// the transition logic only ever sees the specific facts it's allowed
/// to depend on.
pub struct PhaseInputs<'a> {
    pub altitude_ft: f64,
    pub runway_elevation_ft: f64,
    pub engine_n1_percent: f64,
    pub atc: &'a Controller,
}

pub struct PhaseTracker {
    current: FlightPhase,
    /// Altitude when the current go-around began, if we're in one --
    /// used to require an actual altitude *gain* before calling it
    /// complete, rather than just "above the ground margin" (which
    /// would resolve to Climb on the very next tick, since go-arounds
    /// start well above that margin already).
    go_around_started_at_ft: Option<f64>,
}

impl PhaseTracker {
    pub fn new() -> Self {
        Self { current: FlightPhase::Ground, go_around_started_at_ft: None }
    }

    pub fn current(&self) -> FlightPhase {
        self.current
    }

    /// Check whether the current phase's automatic transition condition
    /// is met, and advance if so. Called once per tick from
    /// `Simulation::tick`. Returns the new phase if a transition
    /// happened this tick.
    pub fn tick(&mut self, inputs: &PhaseInputs) -> Option<FlightPhase> {
        let next = match self.current {
            FlightPhase::Ground => {
                inputs.atc.is_granted(ClearanceType::Taxi).then_some(FlightPhase::Taxi)
            }

            FlightPhase::Taxi => {
                inputs.atc.is_granted(ClearanceType::Takeoff).then_some(FlightPhase::TakeoffHold)
            }

            FlightPhase::TakeoffHold => (inputs.atc.is_granted(ClearanceType::Takeoff)
                && inputs.engine_n1_percent >= TAKEOFF_ROLL_N1_PERCENT)
                .then_some(FlightPhase::Takeoff),

            FlightPhase::Takeoff => (inputs.altitude_ft
                > inputs.runway_elevation_ft + GROUND_MARGIN_FT)
                .then_some(FlightPhase::Climb),

            FlightPhase::Climb => {
                (inputs.altitude_ft >= CRUISE_ALTITUDE_FT).then_some(FlightPhase::Cruise)
            }

            FlightPhase::Cruise => {
                inputs.atc.is_granted(ClearanceType::Descend).then_some(FlightPhase::Descent)
            }

            FlightPhase::Descent => {
                inputs.atc.is_granted(ClearanceType::Approach).then_some(FlightPhase::Approach)
            }

            FlightPhase::Approach => (inputs.atc.is_granted(ClearanceType::Landing)
                && inputs.altitude_ft <= inputs.runway_elevation_ft + LANDING_ALTITUDE_MARGIN_FT)
                .then_some(FlightPhase::Landing),

            FlightPhase::Landing => (inputs.altitude_ft
                <= inputs.runway_elevation_ft + GROUND_MARGIN_FT)
                .then_some(FlightPhase::TaxiToGate),

            FlightPhase::GoAround => {
                let started_at = self.go_around_started_at_ft?;
                (inputs.altitude_ft >= started_at + GO_AROUND_CLIMB_GAIN_FT)
                    .then_some(FlightPhase::Climb)
            }

            // No automatic transition -- TaxiToGate only advances via an
            // explicit SHUTDOWN command, and Shutdown is terminal.
            FlightPhase::TaxiToGate | FlightPhase::Shutdown => None,
        };

        if let Some(phase) = next {
            self.current = phase;
            if phase != FlightPhase::GoAround {
                self.go_around_started_at_ft = None;
            }
        }
        next
    }

    /// Explicit player command: abandon the approach/landing and climb
    /// away. Only makes sense from Approach or Landing -- rejected
    /// (with a reason) from anywhere else, the same way a real go-around
    /// call doesn't make sense mid-cruise.
    pub fn go_around(&mut self, altitude_ft: f64) -> Result<FlightPhase, String> {
        match self.current {
            FlightPhase::Approach | FlightPhase::Landing => {
                self.current = FlightPhase::GoAround;
                self.go_around_started_at_ft = Some(altitude_ft);
                Ok(self.current)
            }
            other => Err(format!(
                "go-around only makes sense during approach or landing, currently {}",
                other.name()
            )),
        }
    }

    /// Explicit player command: shut down after taxiing to the gate.
    /// Only valid from TaxiToGate.
    pub fn shutdown(&mut self) -> Result<FlightPhase, String> {
        match self.current {
            FlightPhase::TaxiToGate => {
                self.current = FlightPhase::Shutdown;
                Ok(self.current)
            }
            other => Err(format!(
                "nothing to shut down -- currently {}, not taxied to gate",
                other.name()
            )),
        }
    }
}

impl Default for PhaseTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inputs<'a>(
        altitude_ft: f64,
        runway_elevation_ft: f64,
        engine_n1_percent: f64,
        atc: &'a Controller,
    ) -> PhaseInputs<'a> {
        PhaseInputs { altitude_ft, runway_elevation_ft, engine_n1_percent, atc }
    }

    #[test]
    fn starts_on_the_ground() {
        assert_eq!(PhaseTracker::new().current(), FlightPhase::Ground);
    }

    #[test]
    fn stays_on_ground_without_a_taxi_clearance() {
        let mut tracker = PhaseTracker::new();
        let atc = Controller::new();
        assert_eq!(tracker.tick(&inputs(620.0, 620.0, 0.0, &atc)), None);
        assert_eq!(tracker.current(), FlightPhase::Ground);
    }

    #[test]
    fn full_departure_sequence_advances_phase_by_phase() {
        let mut tracker = PhaseTracker::new();
        let mut atc = Controller::new();
        let runway_elevation = 620.0;

        // Ground -> Taxi
        atc.request(ClearanceType::Taxi);
        assert_eq!(
            tracker.tick(&inputs(runway_elevation, runway_elevation, 0.0, &atc)),
            Some(FlightPhase::Taxi)
        );

        // Taxi -> TakeoffHold
        atc.request(ClearanceType::Takeoff);
        assert_eq!(
            tracker.tick(&inputs(runway_elevation, runway_elevation, 0.0, &atc)),
            Some(FlightPhase::TakeoffHold)
        );

        // TakeoffHold -> Takeoff: needs the throttle-up too, not just
        // the clearance already granted.
        assert_eq!(tracker.tick(&inputs(runway_elevation, runway_elevation, 10.0, &atc)), None);
        assert_eq!(
            tracker.tick(&inputs(runway_elevation, runway_elevation, 60.0, &atc)),
            Some(FlightPhase::Takeoff)
        );

        // Takeoff -> Climb: needs to actually leave the ground.
        assert_eq!(
            tracker.tick(&inputs(runway_elevation + 5.0, runway_elevation, 90.0, &atc)),
            None
        );
        assert_eq!(
            tracker.tick(&inputs(runway_elevation + 50.0, runway_elevation, 90.0, &atc)),
            Some(FlightPhase::Climb)
        );

        // Climb -> Cruise
        assert_eq!(
            tracker.tick(&inputs(CRUISE_ALTITUDE_FT, runway_elevation, 85.0, &atc)),
            Some(FlightPhase::Cruise)
        );
    }

    #[test]
    fn cruise_to_landing_sequence_follows_atc_and_altitude() {
        let mut tracker = PhaseTracker::new();
        // Fast-forward to Cruise by constructing a tracker already there
        // via the same path a real one would take -- simplest is to
        // drive it through, reusing the departure test's approach at a
        // smaller scale.
        let mut atc = Controller::new();
        atc.request(ClearanceType::Taxi);
        atc.request(ClearanceType::Takeoff);
        let elevation = 620.0;
        tracker.tick(&inputs(elevation, elevation, 0.0, &atc)); // -> Taxi
        tracker.tick(&inputs(elevation, elevation, 0.0, &atc)); // -> TakeoffHold
        tracker.tick(&inputs(elevation, elevation, 60.0, &atc)); // -> Takeoff
        tracker.tick(&inputs(elevation + 50.0, elevation, 90.0, &atc)); // -> Climb
        tracker.tick(&inputs(CRUISE_ALTITUDE_FT, elevation, 85.0, &atc)); // -> Cruise
        assert_eq!(tracker.current(), FlightPhase::Cruise);

        // Cruise -> Descent
        atc.request(ClearanceType::Descend);
        assert_eq!(
            tracker.tick(&inputs(CRUISE_ALTITUDE_FT, elevation, 85.0, &atc)),
            Some(FlightPhase::Descent)
        );

        // Descent -> Approach
        atc.request(ClearanceType::Approach);
        assert_eq!(
            tracker.tick(&inputs(2_000.0, elevation, 60.0, &atc)),
            Some(FlightPhase::Approach)
        );

        // Approach -> Landing needs both the clearance and low altitude.
        atc.request(ClearanceType::Landing);
        assert_eq!(tracker.tick(&inputs(2_000.0, elevation, 40.0, &atc)), None);
        assert_eq!(
            tracker.tick(&inputs(elevation + 200.0, elevation, 30.0, &atc)),
            Some(FlightPhase::Landing)
        );

        // Landing -> TaxiToGate on touchdown.
        assert_eq!(
            tracker.tick(&inputs(elevation, elevation, 0.0, &atc)),
            Some(FlightPhase::TaxiToGate)
        );
    }

    #[test]
    fn go_around_is_rejected_outside_approach_or_landing() {
        let mut tracker = PhaseTracker::new(); // Ground
        let result = tracker.go_around(620.0);
        assert!(result.is_err());
        assert_eq!(tracker.current(), FlightPhase::Ground);
    }

    #[test]
    fn go_around_requires_a_real_altitude_gain_before_resolving_to_climb() {
        let mut tracker = PhaseTracker::new();
        // Force into Approach via the internal field for a focused test
        // rather than replaying the whole departure sequence.
        tracker.current = FlightPhase::Approach;

        let atc = Controller::new();
        let elevation = 620.0;
        let start_altitude = elevation + 300.0;

        assert!(tracker.go_around(start_altitude).is_ok());
        assert_eq!(tracker.current(), FlightPhase::GoAround);

        // Barely climbed -- not enough to resolve yet.
        assert_eq!(tracker.tick(&inputs(start_altitude + 50.0, elevation, 90.0, &atc)), None);
        assert_eq!(tracker.current(), FlightPhase::GoAround);

        // Climbed enough now.
        assert_eq!(
            tracker.tick(&inputs(start_altitude + GO_AROUND_CLIMB_GAIN_FT, elevation, 90.0, &atc)),
            Some(FlightPhase::Climb)
        );
    }

    #[test]
    fn shutdown_is_rejected_before_taxi_to_gate() {
        let mut tracker = PhaseTracker::new();
        assert!(tracker.shutdown().is_err());
    }

    #[test]
    fn shutdown_succeeds_from_taxi_to_gate() {
        let mut tracker = PhaseTracker::new();
        tracker.current = FlightPhase::TaxiToGate;
        assert_eq!(tracker.shutdown(), Ok(FlightPhase::Shutdown));
        assert_eq!(tracker.current(), FlightPhase::Shutdown);
    }
}
