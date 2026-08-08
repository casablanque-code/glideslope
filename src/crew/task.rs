//! A single unit of work the FO can perform.
//!
//! Roadmap examples: "Read QRH", "Readback", "Flaps 15", "Contact
//! Tower", "Landing Checklist". Checklists (#8) are now a real source of
//! tasks; ATC comms/other sources still don't exist.

use crate::core::ids::TaskId;
use std::time::Duration;

/// What a task is tied to, if anything -- lets `Simulation` react when a
/// task completes (e.g. marking a checklist item done) without every
/// caller having to know how completion should be handled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskSource {
    /// Not tied to any tracked system. Completing it has no further
    /// effect beyond the queue itself. No production call site yet
    /// (checklists are the only real task source so far) -- used in
    /// crew::queue's tests, and will get a real one once another task
    /// source exists (ATC comms, ...).
    #[allow(dead_code)]
    Adhoc,
    /// Index into the active checklist's items. When this task
    /// completes, `Simulation` marks that checklist item done.
    ChecklistItem(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: TaskId,
    pub description: String,
    /// How long this task takes a baseline (fully experienced,
    /// unfatigued) FO to complete. The FO's actual completion time
    /// scales this -- see `crew::fo::FirstOfficer::speed_multiplier`.
    pub base_duration: Duration,
    pub source: TaskSource,
}

impl Task {
    pub fn new(
        id: TaskId,
        description: impl Into<String>,
        base_duration: Duration,
        source: TaskSource,
    ) -> Self {
        Self { id, description: description.into(), base_duration, source }
    }
}
