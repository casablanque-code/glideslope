//! Top-level simulation: owns the clock and event bus, drives one tick at
//! a time. Aircraft/crew/world state gets added here as those subsystems
//! land — for now this just proves the tick → event flow works end to end.

use crate::aircraft::controls::ControlInputs;
use crate::aircraft::state::AircraftState;
use crate::checklist::landing::landing_checklist;
use crate::checklist::Checklist;
use crate::core::command::Command;
use crate::core::event::Event;
use crate::core::event_bus::EventBus;
use crate::core::ids::{IdGenerator, TaskId};
use crate::core::tick::FixedTimestep;
use crate::core::time::{SimClock, TICK_DURATION};
use crate::crew::fo::FirstOfficer;
use crate::crew::queue::TaskQueue;
use crate::crew::task::{Task, TaskSource};
use std::time::Duration;

/// How long the FO takes on a delegated checklist item (before their
/// own experience-based speed multiplier is applied). Not sourced from
/// any real checklist-item timing data -- picked to feel deliberate
/// without being tedious to watch.
const CHECKLIST_ITEM_DURATION: Duration = Duration::from_secs(5);

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
    first_officer: FirstOfficer,
    fo_queue: TaskQueue,
    task_ids: IdGenerator,
    checklist: Checklist,
    /// Tracks which checklist items already have a task delegated for
    /// them, separately from `complete` -- without this, calling
    /// "delegate next item" twice before the first finishes would queue
    /// the same item to the FO twice.
    checklist_delegated: Vec<bool>,
}

impl Simulation {
    pub fn new() -> Self {
        let checklist = landing_checklist();
        let checklist_delegated = vec![false; checklist.items.len()];

        Self {
            clock: SimClock::new(),
            event_bus: EventBus::new(),
            timestep: FixedTimestep::new(),
            started: false,
            aircraft: AircraftState::cruise(),
            controls: ControlInputs::default(),
            first_officer: FirstOfficer::default(),
            fo_queue: TaskQueue::new(),
            task_ids: IdGenerator::new(),
            checklist,
            checklist_delegated,
        }
    }

    pub fn clock(&self) -> &SimClock {
        &self.clock
    }

    pub fn aircraft(&self) -> &AircraftState {
        &self.aircraft
    }

    pub fn fo_queue(&self) -> &TaskQueue {
        &self.fo_queue
    }

    pub fn checklist(&self) -> &Checklist {
        &self.checklist
    }

    /// Hand a task to the FO. Mints a fresh `TaskId` so callers don't
    /// need their own id bookkeeping.
    pub fn delegate_task(
        &mut self,
        description: impl Into<String>,
        base_duration: Duration,
        source: TaskSource,
    ) {
        let task = Task::new(TaskId(self.task_ids.next()), description, base_duration, source);
        self.fo_queue.delegate(task, &self.first_officer);
    }

    /// Delegate the next not-yet-complete, not-already-delegated checklist
    /// item to the FO. Returns the item's name, or `None` if there's
    /// nothing left to delegate (checklist complete, or every remaining
    /// item is already in the FO's queue).
    pub fn delegate_next_checklist_item(&mut self) -> Option<String> {
        let idx = self
            .checklist
            .items
            .iter()
            .zip(self.checklist_delegated.iter())
            .position(|(item, delegated)| !item.complete && !*delegated)?;

        self.checklist_delegated[idx] = true;
        let name = self.checklist.items[idx].name.clone();
        self.delegate_task(
            format!("Checklist: {name}"),
            CHECKLIST_ITEM_DURATION,
            TaskSource::ChecklistItem(idx),
        );
        Some(name)
    }

    /// Mark the next pending checklist item complete directly -- the
    /// player performing it themselves instead of delegating to the FO.
    /// Takes effect immediately, no queue/timing involved.
    pub fn check_next_checklist_item(&mut self) -> Option<String> {
        self.checklist.check_next()
    }

    /// Apply a parsed player command by updating the control targets the
    /// aircraft chases on subsequent ticks. Takes effect starting next
    /// tick, not instantly -- `AircraftState::integrate` still applies
    /// its usual first-order response toward the new target.
    pub fn apply_command(&mut self, command: Command) {
        match command {
            Command::Pitch(deg) => self.controls.pitch_target_deg = deg,
            Command::Bank(deg) => self.controls.bank_target_deg = deg,
            Command::Thrust(percent) => self.controls.thrust_target_percent = percent,
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
        if let Some(completed) = self.fo_queue.integrate(TICK_DURATION, &self.first_officer) {
            if let TaskSource::ChecklistItem(idx) = completed.source {
                if let Some(item) = self.checklist.items.get_mut(idx) {
                    item.complete = true;
                }
            }
            self.event_bus.publish(Event::TaskCompleted { task: completed });
        }
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
    use crate::core::time::{TICKS_PER_SECOND, TICK_DURATION};
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
    fn delegated_task_completes_and_publishes_an_event() {
        let mut sim = Simulation::new();
        let completed = Arc::new(Mutex::new(Vec::new()));
        let completed_clone = Arc::clone(&completed);

        sim.event_bus().subscribe(move |event| {
            if let Event::TaskCompleted { task } = event {
                completed_clone.lock().unwrap().push(task.description.clone());
            }
        });

        sim.delegate_task("Read QRH", Duration::from_secs(1), TaskSource::Adhoc);
        assert!(sim.fo_queue().executing().is_some());

        // Default FO isn't fully experienced (see FirstOfficer::default),
        // so a "1 second" task can take up to 2x that -- advance well
        // past the worst case rather than assuming a 1x multiplier.
        for _ in 0..(TICKS_PER_SECOND as usize * 3) {
            sim.tick();
        }

        assert_eq!(*completed.lock().unwrap(), vec!["Read QRH".to_string()]);
        assert!(sim.fo_queue().executing().is_none());
    }

    #[test]
    fn manually_checking_an_item_completes_it_instantly() {
        let mut sim = Simulation::new();
        let checked = sim.check_next_checklist_item();
        assert_eq!(checked, Some("Gear".to_string()));
        assert!(sim.checklist().items[0].complete);
        assert!(sim.fo_queue().executing().is_none()); // no FO involvement
    }

    #[test]
    fn delegating_a_checklist_item_does_not_complete_it_until_the_fo_finishes() {
        let mut sim = Simulation::new();
        let delegated = sim.delegate_next_checklist_item();

        assert_eq!(delegated, Some("Gear".to_string()));
        assert!(!sim.checklist().items[0].complete);
        assert!(sim.fo_queue().executing().is_some());

        for _ in 0..(TICKS_PER_SECOND as usize * 15) {
            sim.tick();
        }

        assert!(sim.checklist().items[0].complete);
    }

    #[test]
    fn delegating_twice_before_completion_queues_two_different_items_not_the_same_one() {
        let mut sim = Simulation::new();
        let first = sim.delegate_next_checklist_item();
        let second = sim.delegate_next_checklist_item();

        // Delegation isn't sequentially gated -- the FO can have more
        // than one item queued at once, same as any other tasks. What
        // it must never do is re-delegate an item already in flight.
        assert_eq!(first, Some("Gear".to_string()));
        assert_eq!(second, Some("Flaps".to_string()));
        assert_eq!(sim.fo_queue().pending().count(), 1);

        // A third call moves on to the next undelegated item still left.
        let third = sim.delegate_next_checklist_item();
        assert_eq!(third, Some("Spoilers".to_string()));
    }

    #[test]
    fn delegating_repeatedly_eventually_exhausts_every_item() {
        let mut sim = Simulation::new();
        let mut delegated = Vec::new();
        while let Some(name) = sim.delegate_next_checklist_item() {
            delegated.push(name);
        }
        assert_eq!(delegated, vec!["Gear", "Flaps", "Spoilers", "Autobrake", "Cabin"]);
        assert_eq!(sim.delegate_next_checklist_item(), None);
    }

    #[test]
    fn checklist_is_complete_once_every_item_is_checked() {
        let mut sim = Simulation::new();
        while sim.check_next_checklist_item().is_some() {}
        assert!(sim.checklist().is_complete());
    }

    #[test]
    fn apply_command_updates_control_targets_and_aircraft_responds_next_ticks() {
        let mut sim = Simulation::new();
        let initial_altitude = sim.aircraft().altitude_ft;

        sim.apply_command(Command::Pitch(7.5));
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
