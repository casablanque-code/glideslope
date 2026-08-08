//! Shared error types for the simulation core.
//!
//! Kept dependency-free (no `thiserror`) while the surface is this small;
//! revisit if/when the number of variants and `From` impls grows enough
//! that hand-written boilerplate becomes a maintenance cost.

use std::fmt;

#[derive(Debug)]
#[allow(dead_code)] // stub type ahead of issue #4/#7 call sites
pub enum SimError {
    /// A command referenced something that doesn't exist yet (e.g. a task
    /// id the FO queue doesn't know about). Placeholder until the parser
    /// and FO queue issues land and give this real call sites.
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
