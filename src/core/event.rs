//! The event vocabulary subsystems use to talk to each other.
//!
//! Per the architecture rule "subsystems never directly manipulate each
//! other" — this enum is the entire surface between them. A subsystem may
//! only publish events describing what happened and react to events it
//! subscribes to; it never reaches into another subsystem's state.
//!
//! This starts small on purpose. Later issues (aircraft state, FO queue,
//! sensors, ...) add variants as those subsystems land — resist the urge
//! to pre-declare events for systems that don't exist yet.

use crate::core::time::SimTime;

#[derive(Debug, Clone)]
pub enum Event {
    /// Published by the sim loop at the end of every tick. Subsystems that
    /// only care about "time passed" (rather than a specific domain event)
    /// subscribe to this.
    Tick { at: SimTime },

    /// Published once when the simulation starts running.
    SimulationStarted,

    /// Published once when the simulation loop exits.
    SimulationStopped { at: SimTime },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_event_carries_the_time_it_fired_at() {
        let event = Event::Tick {
            at: SimTime::from_ticks(42),
        };
        match event {
            Event::Tick { at } => assert_eq!(at.ticks(), 42),
            _ => panic!("expected Tick"),
        }
    }
}
