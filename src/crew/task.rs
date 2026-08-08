//! A single unit of work the FO can perform.
//!
//! Roadmap examples: "Read QRH", "Readback", "Flaps 15", "Contact
//! Tower", "Landing Checklist" -- none of those systems exist yet
//! (checklists are #8, ATC comms don't exist), so this issue only
//! establishes the task/queue mechanic itself, not the full task
//! vocabulary. Tasks are constructed ad hoc (see tests) until something
//! that actually generates them lands.

use crate::core::ids::TaskId;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    /// How long this task takes a baseline (fully experienced,
    /// unfatigued) FO to complete. The FO's actual completion time
    /// scales this -- see `crew::fo::FirstOfficer::speed_multiplier`.
    pub base_duration: Duration,
}

impl Task {
    pub fn new(id: TaskId, description: impl Into<String>, base_duration: Duration) -> Self {
        Self { id, description: description.into(), base_duration }
    }
}
