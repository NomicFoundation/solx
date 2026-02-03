//!
//! The continue statement.
//!

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::keyword::Keyword;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;

///
/// The Yul continue statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Continue {
    /// The location.
    pub location: Location,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl Continue {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::parser::take_or_next(initial, lexer)?;
        let location = token.location;

        let solc_location =
            token
                .take_solidity_location()
                .map_err(|error| ParserError::DebugInfoParseError {
                    location: token.location,
                    details: error.to_string(),
                })?;

        match token {
            Token {
                lexeme: Lexeme::Keyword(Keyword::Continue),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["continue"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        Ok(Self {
            location,
            solc_location,
        })
    }
}
