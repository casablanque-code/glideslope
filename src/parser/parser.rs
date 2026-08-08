//! Turns tokens into a `core::command::Command`, validating against the
//! grammar in `grammar.rs`. Intentionally a flat two-token match
//! (`COMMAND value`) rather than a general recursive-descent parser --
//! there's exactly one shape of command right now. Revisit the structure
//! if a command ever needs more than one argument or sub-clauses.

use super::grammar::{self, CommandSpec};
use super::lexer::Token;
use crate::core::command::Command;
use crate::core::errors::SimError;

pub fn parse_tokens(tokens: &[Token]) -> Result<Command, SimError> {
    let words: Vec<&str> = tokens.iter().map(|Token::Word(word)| word.as_str()).collect();

    match words.as_slice() {
        [] => Err(SimError::InvalidCommand("empty command".to_string())),
        [name] => {
            Err(SimError::InvalidCommand(format!("'{name}' expects a value, e.g. '{name} 5'")))
        }
        [name, value] => {
            let spec = grammar::lookup(name)
                .ok_or_else(|| SimError::InvalidCommand(format!("unknown command '{name}'")))?;
            let parsed: f64 = value
                .parse()
                .map_err(|_| SimError::InvalidCommand(format!("'{value}' is not a number")))?;
            validate_range(spec, parsed)?;
            Ok(build_command(spec, parsed))
        }
        [name, ..] => Err(SimError::InvalidCommand(format!("'{name}' takes exactly one value"))),
    }
}

fn validate_range(spec: &CommandSpec, value: f64) -> Result<(), SimError> {
    if spec.range.contains(&value) {
        Ok(())
    } else {
        Err(SimError::InvalidCommand(format!(
            "{} must be between {} and {}, got {value}",
            spec.name,
            spec.range.start(),
            spec.range.end()
        )))
    }
}

fn build_command(spec: &CommandSpec, value: f64) -> Command {
    match spec.name {
        "PITCH" => Command::Pitch(value),
        "BANK" => Command::Bank(value),
        "THRUST" => Command::Thrust(value),
        // Unreachable as long as `grammar::{PITCH,BANK,THRUST}` and this
        // match stay in sync -- a mismatch here is a bug caught by the
        // tests below, not something a user's input can trigger.
        other => unreachable!("grammar spec '{other}' has no matching Command variant"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::lexer::tokenize;
    use super::*;

    fn parse(input: &str) -> Result<Command, SimError> {
        parse_tokens(&tokenize(input))
    }

    #[test]
    fn every_grammar_command_builds_successfully() {
        assert_eq!(parse("PITCH 0"), Ok(Command::Pitch(0.0)));
        assert_eq!(parse("BANK 0"), Ok(Command::Bank(0.0)));
        assert_eq!(parse("THRUST 0"), Ok(Command::Thrust(0.0)));
    }

    #[test]
    fn range_boundaries_are_inclusive() {
        assert!(parse("THRUST 0").is_ok());
        assert!(parse("THRUST 100").is_ok());
        assert!(parse("THRUST 100.01").is_err());
        assert!(parse("THRUST -0.01").is_err());
    }
}
