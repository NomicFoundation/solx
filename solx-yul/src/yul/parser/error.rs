//!
//! The Yul IR parser error.
//!

use std::collections::BTreeSet;

use crate::yul::lexer::token::location::Location;

///
/// The Yul IR parser error.
///
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    /// An invalid token received from the lexer.
    #[error("{location} Expected one of {expected:?}, found `{found}`")]
    InvalidToken {
        /// The invalid token location.
        location: Location,
        /// The list of expected tokens.
        expected: Vec<&'static str>,
        /// The invalid token.
        found: String,
    },
    /// A reserved keyword cannot be used as an identifier.
    #[error("{location} The identifier `{identifier}` is reserved")]
    ReservedIdentifier {
        /// The invalid token location.
        location: Location,
        /// The invalid identifier.
        identifier: String,
    },
    /// Invalid number of function arguments.
    #[error("{location} Function `{identifier}` must have {expected} arguments, found {found}")]
    InvalidNumberOfArguments {
        /// The invalid function location.
        location: Location,
        /// The invalid function name.
        identifier: String,
        /// The expected number of arguments.
        expected: usize,
        /// The actual number of arguments.
        found: usize,
    },
    /// Invalid attributes.
    #[error("{location} Found invalid LLVM attributes: {values:?}")]
    InvalidAttributes {
        /// The invalid token location.
        location: Location,
        /// The list of invalid attributes.
        values: BTreeSet<String>,
    },
    /// Debug info parsing.
    #[error("{location} Failed to parse debug info: {details}")]
    DebugInfoParseError {
        /// The invalid token location.
        location: Location,
        /// The error details.
        details: String,
    },
}
