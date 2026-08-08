//! Tokenizes raw command-line input into words the parser can match
//! against grammar rules. Deliberately dumb -- no quoting, no operators,
//! just whitespace-separated words -- because the grammar itself is
//! currently just `COMMAND value` (see `grammar.rs`). Revisit if a
//! command ever needs quoted strings or multiple word-like arguments.

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
}

pub fn tokenize(input: &str) -> Vec<Token> {
    input.split_whitespace().map(|word| Token::Word(word.to_string())).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_whitespace() {
        let tokens = tokenize("PITCH 5.0");
        assert_eq!(tokens, vec![Token::Word("PITCH".into()), Token::Word("5.0".into())]);
    }

    #[test]
    fn collapses_repeated_and_leading_whitespace() {
        let tokens = tokenize("  BANK   -10  ");
        assert_eq!(tokens, vec![Token::Word("BANK".into()), Token::Word("-10".into())]);
    }

    #[test]
    fn empty_input_yields_no_tokens() {
        assert_eq!(tokenize("   "), Vec::new());
        assert_eq!(tokenize(""), Vec::new());
    }
}
