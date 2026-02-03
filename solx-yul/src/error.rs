//!
//! The Yul IR error.
//!

use crate::lexer::error::Error as LexerError;
use crate::parser::error::Error as ParserError;

///
/// The Yul IR error.
///
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The lexer error.
    #[error("Lexical: {0}")]
    Lexer(#[from] LexerError),
    /// The parser error.
    #[error("Syntax: {0}")]
    Parser(#[from] ParserError),
}
