//!
//! The single-line comment lexeme.
//!

use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::location::Location;

///
/// The single-line comment lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {}

impl Comment {
    /// The start symbol.
    pub const START: &'static str = "//";
    /// The end symbol.
    pub const END: &'static str = "\n";

    ///
    /// Returns the comment's length, including the trimmed whitespace around it.
    ///
    pub fn parse(input: &str) -> Token {
        let end_position = input.find(Self::END).unwrap_or(input.len());
        let length = end_position + Self::END.len();

        Token::new(Location::new(1, 1), Lexeme::Comment, length)
    }
}
