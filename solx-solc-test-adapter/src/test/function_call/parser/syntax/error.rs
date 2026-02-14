//!
//! The syntax parser error.
//!

use crate::test::function_call::parser::lexical::Error as LexicalError;
use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Location;

///
/// The syntax parser error.
///
#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    /// The invalid lexeme location.
    pub location: Location,
    /// The list of the expected lexemes.
    pub expected: String,
    /// The invalid lexeme.
    pub found: Lexeme,
}

///
/// The lexical and syntax errors wrapper.
///
#[derive(Debug, PartialEq, Eq)]
pub enum ParsingError {
    /// The lexical analysis error.
    Lexical(LexicalError),
    /// The syntax analysis error.
    Syntax(Error),
    /// Internal parser state error.
    Builder(String),
}

impl From<LexicalError> for ParsingError {
    fn from(inner: LexicalError) -> Self {
        Self::Lexical(inner)
    }
}

impl From<Error> for ParsingError {
    fn from(inner: Error) -> Self {
        Self::Syntax(inner)
    }
}

impl From<anyhow::Error> for ParsingError {
    fn from(inner: anyhow::Error) -> Self {
        Self::Builder(inner.to_string())
    }
}

impl Error {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(location: Location, expected: Vec<&'static str>, found: Lexeme) -> Self {
        Self {
            location,
            expected: Self::format_one_of(expected.as_slice()),
            found,
        }
    }

    ///
    /// Converts a group of lexemes into a comma-separated list.
    ///
    /// E.g. ["ether", "wei"] turns into `ether`, `wei`.
    ///
    pub fn format_one_of(lexemes: &[&'static str]) -> String {
        lexemes
            .iter()
            .map(|lexeme| format!("`{lexeme}`"))
            .collect::<Vec<String>>()
            .join(", ")
    }
}
