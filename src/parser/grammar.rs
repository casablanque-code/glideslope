//! The command vocabulary itself: which command names exist and what
//! range their single numeric argument is allowed to fall in. Kept
//! separate from `parser.rs` so the vocabulary (what commands mean)
//! doesn't get tangled with the matching logic (how tokens become a
//! `Command`) -- adding a command should mean touching a table here and
//! a match arm in `parser.rs`, not rewriting control flow.
//!
//! Ranges below are operational limits for this fictional aircraft, not
//! sourced from any real type's flight manual -- picked to be generous
//! enough not to get in the way while still rejecting obviously wrong
//! input (e.g. `THRUST 500`).

use std::ops::RangeInclusive;

pub struct CommandSpec {
    pub name: &'static str,
    pub range: RangeInclusive<f64>,
}

pub const PITCH: CommandSpec = CommandSpec { name: "PITCH", range: -15.0..=15.0 };
pub const BANK: CommandSpec = CommandSpec { name: "BANK", range: -30.0..=30.0 };
pub const THRUST: CommandSpec = CommandSpec { name: "THRUST", range: 0.0..=100.0 };

const ALL: [&CommandSpec; 3] = [&PITCH, &BANK, &THRUST];

pub fn lookup(name: &str) -> Option<&'static CommandSpec> {
    ALL.into_iter().find(|spec| spec.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_is_case_insensitive() {
        assert!(lookup("pitch").is_some());
        assert!(lookup("PiTcH").is_some());
    }

    #[test]
    fn lookup_returns_none_for_unknown_command() {
        assert!(lookup("FLAPS").is_none());
    }
}
