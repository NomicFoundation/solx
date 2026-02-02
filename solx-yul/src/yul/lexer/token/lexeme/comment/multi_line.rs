//!
//! The multi-line comment lexeme.
//!

use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::location::Location;

///
/// The multi-line comment lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {}

impl Comment {
    /// The start symbol.
    pub const START: &'static str = "/*";
    /// The end symbol.
    pub const END: &'static str = "*/";

    ///
    /// Returns the comment, including its length and number of lines.
    ///
    pub fn parse(input: &str) -> Token {
        let end_position = input.find(Self::END).unwrap_or(input.len());
        let input = &input[..end_position];

        let length = end_position + Self::END.len();
        let lines = input.matches('\n').count();
        let columns = match input.rfind('\n') {
            Some(new_line) => end_position - (new_line + 1),
            None => end_position,
        };

        Token::new(Location::new(lines, columns), Lexeme::Comment, length)
    }
}
