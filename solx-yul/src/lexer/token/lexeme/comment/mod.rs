//!
//! The comment lexeme.
//!

pub mod multi_line;
pub mod single_line;

use self::multi_line::Comment as MultiLineComment;
use self::single_line::Comment as SingleLineComment;

use crate::lexer::token::Token;

///
/// The comment lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Comment {}

impl Comment {
    ///
    /// Returns the comment's length, including the trimmed whitespace around it.
    ///
    pub fn parse(input: &str) -> Option<Token> {
        if input.starts_with(SingleLineComment::START) {
            Some(SingleLineComment::parse(input))
        } else if input.starts_with(MultiLineComment::START) {
            Some(MultiLineComment::parse(input))
        } else {
            None
        }
    }
}
