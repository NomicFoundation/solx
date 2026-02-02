//!
//! The for-loop statement.
//!

use std::collections::BTreeSet;

use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::keyword::Keyword;
use crate::yul::lexer::token::location::Location;
use crate::yul::parser::dialect::Dialect;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::statement::block::Block;
use crate::yul::parser::statement::expression::Expression;

///
/// The Yul for-loop statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(bound = "P: serde::de::DeserializeOwned")]
pub struct ForLoop<P>
where
    P: Dialect,
{
    /// The location.
    pub location: Location,
    /// The index variables initialization block.
    pub initializer: Block<P>,
    /// The continue condition block.
    pub condition: Expression,
    /// The index variables mutating block.
    pub finalizer: Block<P>,
    /// The loop body.
    pub body: Block<P>,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl<P> ForLoop<P>
where
    P: Dialect,
{
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let mut token = crate::yul::parser::take_or_next(initial, lexer)?;
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
                lexeme: Lexeme::Keyword(Keyword::For),
                ..
            } => {}
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["for"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let initializer = Block::parse(lexer, None)?;

        let condition = Expression::parse(lexer, None)?;

        let finalizer = Block::parse(lexer, None)?;

        let body = Block::parse(lexer, None)?;

        Ok(Self {
            location,
            initializer,
            condition,
            finalizer,
            body,
            solc_location,
        })
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        let mut libraries = self.initializer.get_unlinked_libraries();
        libraries.extend(self.condition.get_unlinked_libraries());
        libraries.extend(self.finalizer.get_unlinked_libraries());
        libraries.extend(self.body.get_unlinked_libraries());
        libraries
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        self.initializer.accumulate_evm_dependencies(dependencies);
        self.condition.accumulate_evm_dependencies(dependencies);
        self.finalizer.accumulate_evm_dependencies(dependencies);
        self.body.accumulate_evm_dependencies(dependencies);
    }
}
