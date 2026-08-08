pub mod grammar;
pub mod lexer;
pub mod parser;

use crate::core::command::Command;
use crate::core::errors::SimError;

/// Parse a raw line of player input into a `Command`, or a `SimError`
/// describing what's wrong with it. The only entry point external callers
/// (the app's command line) should use -- lexer/parser/grammar internals
/// stay private to this module's own wiring.
pub fn parse(input: &str) -> Result<Command, SimError> {
    let tokens = lexer::tokenize(input);
    parser::parse_tokens(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_pitch_command() {
        assert_eq!(parse("PITCH 5"), Ok(Command::SetPitch(5.0)));
    }

    #[test]
    fn parses_valid_bank_command_with_negative_value() {
        assert_eq!(parse("BANK -10"), Ok(Command::SetBank(-10.0)));
    }

    #[test]
    fn parses_valid_thrust_command() {
        assert_eq!(parse("THRUST 90"), Ok(Command::SetThrust(90.0)));
    }

    #[test]
    fn command_name_is_case_insensitive() {
        assert_eq!(parse("bank -10"), Ok(Command::SetBank(-10.0)));
        assert_eq!(parse("Thrust 50"), Ok(Command::SetThrust(50.0)));
    }

    #[test]
    fn tolerates_extra_surrounding_whitespace() {
        assert_eq!(parse("   PITCH   5   "), Ok(Command::SetPitch(5.0)));
    }

    #[test]
    fn rejects_unknown_command() {
        assert!(parse("FLAPS 15").is_err());
    }

    #[test]
    fn rejects_out_of_range_value() {
        assert!(parse("THRUST 150").is_err());
    }

    #[test]
    fn rejects_non_numeric_value() {
        assert!(parse("PITCH up").is_err());
    }

    #[test]
    fn rejects_missing_value() {
        assert!(parse("PITCH").is_err());
    }

    #[test]
    fn rejects_too_many_values() {
        assert!(parse("PITCH 5 10").is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(parse("").is_err());
    }

    #[test]
    fn rejects_whitespace_only_input() {
        assert!(parse("   ").is_err());
    }
}
