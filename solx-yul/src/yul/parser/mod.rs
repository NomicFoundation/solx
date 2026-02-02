//!
//! The Yul code block.
//!

pub mod debug_info;
pub mod dialect;
pub mod error;
pub mod identifier;
pub mod statement;
pub mod r#type;

use crate::yul::lexer::Lexer;
use crate::yul::lexer::error::Error as LexerError;
use crate::yul::lexer::token::Token;

///
/// Returns the `token` value if it is `Some(_)`, otherwise takes the next token from the `stream`.
///
pub fn take_or_next(mut token: Option<Token>, lexer: &mut Lexer) -> Result<Token, LexerError> {
    match token.take() {
        Some(token) => Ok(token),
        None => lexer.next(),
    }
}
