//!
//! The assignment expression statement.
//!

use std::collections::BTreeSet;

use crate::yul::error::Error;
use crate::yul::lexer::token::lexeme::symbol::Symbol;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::location::Location;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::Lexer;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::identifier::Identifier;
use crate::yul::parser::statement::expression::Expression;

///
/// The Yul assignment expression statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct Assignment {
    /// The location.
    pub location: Location,
    /// The variable bindings.
    pub bindings: Vec<Identifier>,
    /// The initializing expression.
    pub initializer: Expression,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl Assignment {
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

        let (location, identifier) = match token {
            Token {
                location,
                lexeme: Lexeme::Identifier(identifier),
                ..
            } => (location, identifier),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{identifier}"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };
        let length = identifier.inner.len();

        match lexer.peek()? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Assignment),
                ..
            } => {
                lexer.next()?;

                Ok(Self {
                    location,
                    bindings: vec![Identifier::new(location, identifier.inner)],
                    initializer: Expression::parse(lexer, None)?,
                    solc_location,
                })
            }
            Token {
                lexeme: Lexeme::Symbol(Symbol::Comma),
                ..
            } => {
                let (identifiers, next) = Identifier::parse_list(
                    lexer,
                    Some(Token::new(location, Lexeme::Identifier(identifier), length)),
                )?;

                match crate::yul::parser::take_or_next(next, lexer)? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Assignment),
                        ..
                    } => {}
                    token => {
                        return Err(ParserError::InvalidToken {
                            location: token.location,
                            expected: vec![":="],
                            found: token.lexeme.to_string(),
                        }
                        .into());
                    }
                }

                Ok(Self {
                    location,
                    bindings: identifiers,
                    initializer: Expression::parse(lexer, None)?,
                    solc_location,
                })
            }
            token => Err(ParserError::InvalidToken {
                location: token.location,
                expected: vec![":=", ","],
                found: token.lexeme.to_string(),
            }
            .into()),
        }
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        self.initializer.get_unlinked_libraries()
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        self.initializer.accumulate_evm_dependencies(dependencies);
    }
}
