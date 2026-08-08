//! Typed identifiers for simulation entities.
//!
//! Using distinct newtypes instead of raw `u64`s prevents mixing up, say, a
//! `SensorId` with a `TaskId` at a call site — the compiler catches it.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonically increasing counter used to hand out fresh ids.
///
/// One `IdGenerator` is expected per id "namespace" (sensors, tasks, ...),
/// not one global generator for everything.
#[derive(Debug, Default)]
pub struct IdGenerator(AtomicU64);

impl IdGenerator {
    pub const fn new() -> Self {
        Self(AtomicU64::new(0))
    }

    pub fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}

macro_rules! typed_id {
    ($name:ident) => {
        // SensorId isn't constructed until issue #10 (sensor failures);
        // EntityId has no call site yet either. TaskId is real as of
        // #7. Shared across all three since the allow is harmless on
        // code that's actually used.
        #[allow(dead_code)]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(pub u64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }
    };
}

typed_id!(SensorId);
typed_id!(TaskId);
typed_id!(EntityId);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_increment() {
        let gen = IdGenerator::new();
        assert_eq!(gen.next(), 0);
        assert_eq!(gen.next(), 1);
        assert_eq!(gen.next(), 2);
    }

    #[test]
    fn typed_ids_are_distinct_types() {
        let sensor = SensorId(1);
        let task = TaskId(1);
        // Same underlying value, different types — this is the point.
        assert_eq!(sensor.0, task.0);
    }
}
