//! Player command vocabulary.
//!
//! Stub for now — the real grammar (`FLAPS 15`, `TUNE ILS 109.5`, ...) is
//! issue #4 (command parser). This exists so [`crate::sim::simulation::Simulation`]
//! has a concrete type to accept, instead of every caller inventing its own
//! placeholder ahead of that issue landing.

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // stub type ahead of issue #4 (command parser)
pub enum Command {
    /// No-op, used in tests and as the default before real commands exist.
    Noop,
}
