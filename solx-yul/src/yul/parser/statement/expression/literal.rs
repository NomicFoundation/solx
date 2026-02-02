//!
//! The Yul source code literal.
//!

use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::literal::Literal as LexicalLiteral;
use crate::yul::lexer::token::lexeme::symbol::Symbol;
use crate::yul::lexer::token::location::Location;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::r#type::Type;

///
/// Represents a literal in Yul without differentiating its type.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Literal {
    /// The location.
    pub location: Location,
    /// The lexical literal.
    pub inner: LexicalLiteral,
    /// The type, if it has been explicitly specified.
    pub yul_type: Option<Type>,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl Literal {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::yul::parser::take_or_next(initial, lexer)?;

        let solc_location =
            token
                .take_solidity_location()
                .map_err(|error| ParserError::DebugInfoParseError {
                    location: token.location,
                    details: error.to_string(),
                })?;

        let (location, literal) = match token {
            Token {
                lexeme: Lexeme::Literal(literal),
                location,
                ..
            } => (location, literal),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{literal}"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };

        let yul_type = match lexer.peek()? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Colon),
                ..
            } => {
                lexer.next()?;
                Some(Type::parse(lexer, None)?)
            }
            _ => None,
        };

        Ok(Self {
            location,
            inner: literal,
            yul_type,
            solc_location,
        })
    }
}
