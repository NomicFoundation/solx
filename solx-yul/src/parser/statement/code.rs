//!
//! The Yul code.
//!

use std::collections::BTreeSet;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::block::Block;

///
/// The Yul code entity, which is the first block of the object.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Code {
    /// The location.
    pub location: Location,
    /// The main block.
    pub block: Block,
}

impl Code {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let token = crate::parser::take_or_next(initial, lexer)?;

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

impl solx_codegen_evm::WriteLLVM for Code {
    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        self.block.into_llvm(context)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::lexer::token::location::Location;
    use crate::parser::error::Error;
    use crate::parser::statement::object::Object;

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
        let result = Object::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
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
