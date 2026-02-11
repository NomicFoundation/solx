//!
//! The Yul source code identifier.
//!

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::symbol::Symbol;
use crate::lexer::token::location::Location;

///
/// The Yul source code identifier.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Identifier {
    /// The location.
    pub location: Location,
    /// The inner string.
    pub inner: String,
}

impl Identifier {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(location: Location, inner: String) -> Self {
        Self { location, inner }
    }

    ///
    /// Parses the identifier list where the types cannot be specified.
    ///
    pub fn parse_list(
        lexer: &mut Lexer,
        mut initial: Option<Token>,
    ) -> Result<(Vec<Self>, Option<Token>), Error> {
        let mut result = Vec::new();

        let mut expected_comma = false;
        loop {
            let token = crate::parser::take_or_next(initial.take(), lexer)?;

            match token {
                Token {
                    location,
                    lexeme: Lexeme::Identifier(identifier),
                    ..
                } if !expected_comma => {
                    result.push(Self::new(location, identifier.inner));
                    expected_comma = true;
                }
                Token {
                    lexeme: Lexeme::Symbol(Symbol::Comma),
                    ..
                } if expected_comma => {
                    expected_comma = false;
                }
                token => return Ok((result, Some(token))),
            }
        }
    }
}
