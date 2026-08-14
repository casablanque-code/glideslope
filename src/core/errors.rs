//! Shared error types for the simulation core.
//!
//! Kept dependency-free (no `thiserror`) while the surface is this small;
//! revisit if/when the number of variants and `From` impls grows enough
//! that hand-written boilerplate becomes a maintenance cost.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SimError {
    /// A command referenced something invalid: unknown command name,
    /// wrong number of arguments, a non-numeric value, or a value outside
    /// the command's allowed range. Constructed by `parser::parse`.
    InvalidCommand(String),
    /// A requested flight-phase transition doesn't make sense from the
    /// current phase (e.g. GO AROUND while parked at the gate).
    /// Deliberately not named with an "Invalid" prefix like the variant
    /// above -- clippy::enum_variant_names flags a shared prefix across
    /// all variants (see the app/app.rs and parser/parser.rs history for
    /// the same lint).
    PhaseTransitionRejected(String),
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimError::InvalidCommand(msg) => write!(f, "invalid command: {msg}"),
            SimError::PhaseTransitionRejected(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for SimError {}
