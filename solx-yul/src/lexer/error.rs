//!
//! The Yul IR lexer error.
//!

use crate::lexer::token::location::Location;

///
/// The Yul IR lexer error.
///
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// The invalid lexeme error.
    #[error("{location} Invalid character sequence `{sequence}`")]
    InvalidLexeme {
        /// The lexeme location.
        location: Location,
        /// The invalid sequence of characters.
        sequence: String,
    },
}
