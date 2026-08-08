//! Top-level simulation: owns the clock and event bus, drives one tick at
//! a time. Aircraft/crew/world state gets added here as those subsystems
//! land — for now this just proves the tick → event flow works end to end.

use crate::aircraft::controls::ControlInputs;
use crate::aircraft::state::AircraftState;
use crate::core::command::Command;
use crate::core::event::Event;
use crate::core::event_bus::EventBus;
use crate::core::tick::FixedTimestep;
use crate::core::time::{SimClock, TICK_DURATION};
use std::time::Duration;

pub struct Simulation {
    clock: SimClock,
    event_bus: EventBus,
    timestep: FixedTimestep,
    started: bool,
    aircraft: AircraftState,
    /// No parser (#4) or FO (#7) writes to this yet -- it stays at
    /// `ControlInputs::default()` (cruise trim), which is exactly why
    /// `AircraftState::cruise()` is defined to be in equilibrium with it.
    controls: ControlInputs,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            clock: SimClock::new(),
            event_bus: EventBus::new(),
            timestep: FixedTimestep::new(),
            started: false,
            aircraft: AircraftState::cruise(),
            controls: ControlInputs::default(),
        }
    }

    pub fn clock(&self) -> &SimClock {
        &self.clock
    }

    pub fn aircraft(&self) -> &AircraftState {
        &self.aircraft
    }

    /// Apply a parsed player command by updating the control targets the
    /// aircraft chases on subsequent ticks. Takes effect starting next
    /// tick, not instantly -- `AircraftState::integrate` still applies
    /// its usual first-order response toward the new target.
    pub fn apply_command(&mut self, command: Command) {
        match command {
            Command::SetPitch(deg) => self.controls.pitch_target_deg = deg,
            Command::SetBank(deg) => self.controls.bank_target_deg = deg,
            Command::SetThrust(percent) => self.controls.thrust_target_percent = percent,
        }
    }

    pub fn event_bus(&mut self) -> &mut EventBus {
        &mut self.event_bus
    }

    /// Advance the simulation by exactly one tick: bump the clock, publish
    /// `Event::Tick`. Called directly by callers driving ticks themselves
    /// (tests, a fixed loop count); real-time callers should use
    /// [`Simulation::advance`] instead.
    pub fn tick(&mut self) {
        if !self.started {
            self.event_bus.publish(Event::SimulationStarted);
            self.started = true;
        }
        self.clock.tick();
        self.aircraft.integrate(&self.controls, TICK_DURATION.as_secs_f64());
        self.event_bus.publish(Event::Tick { at: self.clock.now() });
    }

    /// Advance the simulation by `elapsed` wall-clock time, running zero or
    /// more whole ticks via the fixed-timestep accumulator. This is what a
    /// real-time loop (the eventual Ratatui UI loop) should call every
    /// frame instead of calling `tick()` directly.
    pub fn advance(&mut self, elapsed: Duration) {
        self.timestep.advance(elapsed);
        while self.timestep.step() {
            self.tick();
        }
    }

    pub fn stop(&mut self) {
        self.event_bus.publish(Event::SimulationStopped { at: self.clock.now() });
    }
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::time::TICK_DURATION;
    use std::sync::{Arc, Mutex};

    #[test]
    fn tick_advances_clock_and_publishes_event() {
        let mut sim = Simulation::new();
        let ticks_seen = Arc::new(Mutex::new(0));
        let ticks_seen_clone = Arc::clone(&ticks_seen);

        sim.event_bus().subscribe(move |event| {
            if let Event::Tick { .. } = event {
                *ticks_seen_clone.lock().unwrap() += 1;
            }
        });

        sim.tick();
        sim.tick();
        sim.tick();

        assert_eq!(sim.clock().tick_count(), 3);
        assert_eq!(*ticks_seen.lock().unwrap(), 3);
    }

    #[test]
    fn apply_command_updates_control_targets_and_aircraft_responds_next_ticks() {
        let mut sim = Simulation::new();
        let initial_altitude = sim.aircraft().altitude_ft;

        sim.apply_command(Command::SetPitch(7.5));
        for _ in 0..50 {
            sim.tick();
        }

        assert!(sim.aircraft().altitude_ft > initial_altitude);
    }

    #[test]
    fn simulation_started_fires_exactly_once() {
        let mut sim = Simulation::new();
        let starts = Arc::new(Mutex::new(0));
        let starts_clone = Arc::clone(&starts);

        sim.event_bus().subscribe(move |event| {
            if let Event::SimulationStarted = event {
                *starts_clone.lock().unwrap() += 1;
            }
        });

        sim.tick();
        sim.tick();

        assert_eq!(*starts.lock().unwrap(), 1);
    }

    #[test]
    fn advance_runs_whole_ticks_worth_of_elapsed_time() {
        let mut sim = Simulation::new();
        sim.advance(TICK_DURATION * 3 + TICK_DURATION / 2);
        // 3 full ticks consumed; half a tick remains in the accumulator.
        assert_eq!(sim.clock().tick_count(), 3);
    }
}
