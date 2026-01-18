//!
//! The Yul code.
//!

use std::collections::BTreeSet;

use crate::yul::error::Error;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::location::Location;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::Lexer;
use crate::yul::parser::dialect::Dialect;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::statement::block::Block;

///
/// The Yul code entity, which is the first block of the object.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(bound = "P: serde::de::DeserializeOwned")]
pub struct Code<P>
where
    P: Dialect,
{
    /// The location.
    pub location: Location,
    /// The main block.
    pub block: Block<P>,
}

impl<P> Code<P>
where
    P: Dialect,
{
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let token = crate::yul::parser::take_or_next(initial, lexer)?;

        let location = match token {
            Token {
                lexeme: Lexeme::Identifier(identifier),
                location,
                ..
            } if identifier.inner.as_str() == "code" => location,
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["code"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };

        let block = Block::parse(lexer, None)?;

        Ok(Self { location, block })
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        self.block.get_unlinked_libraries()
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        self.block.accumulate_evm_dependencies(dependencies);
    }
}

#[cfg(test)]
mod tests {
    use crate::yul::lexer::token::location::Location;
    use crate::yul::lexer::Lexer;
    use crate::yul::parser::dialect::DefaultDialect;
    use crate::yul::parser::error::Error;
    use crate::yul::parser::statement::object::Object;

    #[test]
    fn error_invalid_token_code() {
        let input = r#"
object "Test" {
    data {
        {
            return(0, 0)
        }
    }
    object "Test_deployed" {
        code {
            {
                return(0, 0)
            }
        }
    }
}
    "#;

        let mut lexer = Lexer::new(input);
        let result =
            Object::<DefaultDialect>::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert_eq!(
            result,
            Err(Error::InvalidToken {
                location: Location::new(3, 5),
                expected: vec!["code"],
                found: "data".to_owned(),
            }
            .into())
        );
    }
}
