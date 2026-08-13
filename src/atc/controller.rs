//! Tracks the state of each clearance type.
//!
//! Roadmap: "basic ATC." Every request is granted immediately today --
//! there's no traffic or weather model yet to generate a real reason to
//! hold or deny one (see DESIGN.md: transitions must follow from real
//! events, never an invisible timer, so a fake "wait 5 minutes" isn't an
//! honest substitute). The request/grant mechanic is real; the "no" case
//! is future work once there's a system that can justify it.

use super::constraints::ClearanceType;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearanceState {
    NotRequested,
    Granted,
}

#[derive(Debug, Clone, Default)]
pub struct Controller {
    clearances: HashMap<ClearanceType, ClearanceState>,
}

impl Controller {
    pub fn new() -> Self {
        Self::default()
    }

    /// Request a clearance. Always grants it today -- see the module
    /// doc. Returns the resulting state so callers don't have to make a
    /// separate call to check what happened.
    pub fn request(&mut self, clearance: ClearanceType) -> ClearanceState {
        self.clearances.insert(clearance, ClearanceState::Granted);
        ClearanceState::Granted
    }

    pub fn state(&self, clearance: ClearanceType) -> ClearanceState {
        self.clearances.get(&clearance).copied().unwrap_or(ClearanceState::NotRequested)
    }

    pub fn is_granted(&self, clearance: ClearanceType) -> bool {
        self.state(clearance) == ClearanceState::Granted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clearance_starts_not_requested() {
        let controller = Controller::new();
        assert_eq!(controller.state(ClearanceType::Taxi), ClearanceState::NotRequested);
        assert!(!controller.is_granted(ClearanceType::Taxi));
    }

    #[test]
    fn requesting_a_clearance_grants_it() {
        let mut controller = Controller::new();
        let result = controller.request(ClearanceType::Takeoff);
        assert_eq!(result, ClearanceState::Granted);
        assert!(controller.is_granted(ClearanceType::Takeoff));
    }

    #[test]
    fn clearances_are_tracked_independently() {
        let mut controller = Controller::new();
        controller.request(ClearanceType::Pushback);
        assert!(controller.is_granted(ClearanceType::Pushback));
        assert!(!controller.is_granted(ClearanceType::Taxi));
    }
}
