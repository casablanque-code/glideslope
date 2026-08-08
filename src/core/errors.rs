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
}

impl fmt::Display for SimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimError::InvalidCommand(msg) => write!(f, "invalid command: {msg}"),
        }
    }
}

impl std::error::Error for SimError {}
