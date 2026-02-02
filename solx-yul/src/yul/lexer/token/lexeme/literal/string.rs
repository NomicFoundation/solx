//!
//! The string literal lexeme.
//!

use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::Literal;
use crate::yul::lexer::token::location::Location;

///
/// The string literal lexeme.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct String {
    /// The inner string contents.
    pub inner: std::string::String,
    /// Whether the string is hexadecimal.
    pub is_hexadecimal: bool,
}

impl String {
    ///
    /// Creates a string literal value.
    ///
    pub fn new(inner: ::std::string::String, is_hexadecimal: bool) -> Self {
        Self {
            inner,
            is_hexadecimal,
        }
    }

    ///
    /// Parses the value from the source code slice.
    ///
    pub fn parse(input: &str) -> Option<Token> {
        let (is_hex_string, mut length, terminator) = if input.starts_with(r#"""#) {
            (false, r#"""#.len(), r#"""#)
        } else if input.starts_with(r#"hex""#) {
            (true, r#"hex""#.len(), r#"""#)
        } else if input.starts_with(r#"\""#) {
            (false, r#"\""#.len(), r#"\""#)
        } else {
            return None;
        };

        let mut string = std::string::String::new();
        loop {
            if input[length..].starts_with('\\') {
                string.push_str(&input[length..length + 2]);
                length += 2;
                continue;
            }

            if input[length..].starts_with(terminator) {
                length += terminator.len();
                break;
            }

            string.push_str(&input[length..length + 1]);
            length += 1;
        }

        let literal = Self::new(string, is_hex_string);

        Some(Token::new(
            Location::new(0, length),
            Lexeme::Literal(Literal::String(literal)),
            length,
        ))
    }
}

impl std::fmt::Display for String {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
