//! Player command vocabulary.
//!
//! Built by `parser::parse` from raw command-line input and applied to
//! `sim::Simulation` via `Simulation::apply_command`. One variant per
//! control the aircraft actually has (see `aircraft::controls`) -- no
//! commands for systems that don't exist yet (flaps, nav radios, ...).

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Pitch(f64),
    Bank(f64),
    Thrust(f64),
}
