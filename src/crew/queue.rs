//! FO task queue: one task executing at a time, the rest waiting in
//! order. Roadmap: "The FO executes one task at a time," with the player
//! deciding whether to interrupt, delegate more, or do it themselves.
//!
//! This issue provides the queue mechanic and completion timing. It does
//! not yet provide a way to interrupt the executing task (there's no
//! command for it -- would need a parser grammar addition once there's
//! a reason to add one) or anything that generates real tasks
//! (checklists are #8, ATC comms don't exist).

use super::fo::FirstOfficer;
use super::task::Task;
use std::collections::VecDeque;
use std::time::Duration;

/// A task currently being worked, with how much time remains at the
/// FO's actual (experience-scaled) pace.
#[derive(Debug, Clone, PartialEq)]
pub struct Executing {
    pub task: Task,
    pub remaining: Duration,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TaskQueue {
    executing: Option<Executing>,
    pending: VecDeque<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a task to the back of the pending queue. If nothing is
    /// currently executing, it starts immediately.
    pub fn delegate(&mut self, task: Task, fo: &FirstOfficer) {
        self.pending.push_back(task);
        self.start_next_if_idle(fo);
    }

    pub fn executing(&self) -> Option<&Executing> {
        self.executing.as_ref()
    }

    pub fn pending(&self) -> impl Iterator<Item = &Task> {
        self.pending.iter()
    }

    /// Advance the executing task's remaining time by `dt`. When it
    /// completes, the next pending task (if any) starts automatically.
    /// Returns the completed task, if one finished this tick.
    pub fn integrate(&mut self, dt: Duration, fo: &FirstOfficer) -> Option<Task> {
        let executing = self.executing.as_mut()?;

        if executing.remaining <= dt {
            let completed = self.executing.take().map(|e| e.task);
            self.start_next_if_idle(fo);
            completed
        } else {
            executing.remaining -= dt;
            None
        }
    }

    fn start_next_if_idle(&mut self, fo: &FirstOfficer) {
        if self.executing.is_none() {
            if let Some(task) = self.pending.pop_front() {
                let remaining = task.base_duration.mul_f64(fo.speed_multiplier());
                self.executing = Some(Executing { task, remaining });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ids::TaskId;

    fn task(id: u64, duration_secs: u64) -> Task {
        Task::new(TaskId(id), format!("task-{id}"), Duration::from_secs(duration_secs))
    }

    #[test]
    fn delegating_to_an_idle_queue_starts_execution_immediately() {
        let mut queue = TaskQueue::new();
        let fo = FirstOfficer::new(1.0);
        queue.delegate(task(1, 10), &fo);

        assert!(queue.executing().is_some());
        assert_eq!(queue.executing().unwrap().task.id, TaskId(1));
        assert_eq!(queue.pending().count(), 0);
    }

    #[test]
    fn a_second_task_waits_in_the_pending_queue() {
        let mut queue = TaskQueue::new();
        let fo = FirstOfficer::new(1.0);
        queue.delegate(task(1, 10), &fo);
        queue.delegate(task(2, 5), &fo);

        assert_eq!(queue.executing().unwrap().task.id, TaskId(1));
        assert_eq!(queue.pending().count(), 1);
        assert_eq!(queue.pending().next().unwrap().id, TaskId(2));
    }

    #[test]
    fn completing_a_task_starts_the_next_pending_one() {
        let mut queue = TaskQueue::new();
        let fo = FirstOfficer::new(1.0);
        queue.delegate(task(1, 1), &fo);
        queue.delegate(task(2, 5), &fo);

        let completed = queue.integrate(Duration::from_secs(1), &fo);

        assert_eq!(completed.map(|t| t.id), Some(TaskId(1)));
        assert_eq!(queue.executing().unwrap().task.id, TaskId(2));
        assert_eq!(queue.pending().count(), 0);
    }

    #[test]
    fn integrate_returns_none_while_a_task_is_still_in_progress() {
        let mut queue = TaskQueue::new();
        let fo = FirstOfficer::new(1.0);
        queue.delegate(task(1, 10), &fo);

        let completed = queue.integrate(Duration::from_secs(1), &fo);

        assert_eq!(completed, None);
        assert!(queue.executing().unwrap().remaining < Duration::from_secs(10));
    }

    #[test]
    fn integrate_on_an_empty_queue_does_nothing_and_does_not_panic() {
        let mut queue = TaskQueue::new();
        let fo = FirstOfficer::new(1.0);
        assert_eq!(queue.integrate(Duration::from_secs(1), &fo), None);
    }

    #[test]
    fn a_less_experienced_fo_takes_longer_to_finish_the_same_task() {
        let mut rookie_queue = TaskQueue::new();
        let mut veteran_queue = TaskQueue::new();
        let rookie = FirstOfficer::new(0.0);
        let veteran = FirstOfficer::new(1.0);

        rookie_queue.delegate(task(1, 10), &rookie);
        veteran_queue.delegate(task(1, 10), &veteran);

        assert!(
            rookie_queue.executing().unwrap().remaining
                > veteran_queue.executing().unwrap().remaining
        );
    }
}
