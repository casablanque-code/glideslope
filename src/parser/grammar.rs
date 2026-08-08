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
    /// One-line explanation shown by the `HELP` command. Kept here
    /// rather than duplicated in `app.rs` so the reference text can
    /// never drift out of sync with the actual validated range.
    pub description: &'static str,
}

pub const PITCH: CommandSpec = CommandSpec {
    name: "PITCH",
    range: -15.0..=15.0,
    description: "target pitch angle, degrees nose up (negative = nose down)",
};
pub const BANK: CommandSpec = CommandSpec {
    name: "BANK",
    range: -30.0..=30.0,
    description: "target bank angle, degrees right (negative = left)",
};
pub const THRUST: CommandSpec = CommandSpec {
    name: "THRUST",
    range: 0.0..=100.0,
    description: "target engine thrust, percent N1",
};

/// Every known command, in the order `HELP` lists them. Exposed publicly
/// so `HELP` can generate its listing from this table instead of a
/// second, hand-maintained copy of the same names/ranges/descriptions.
pub const ALL: [&CommandSpec; 3] = [&PITCH, &BANK, &THRUST];

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
