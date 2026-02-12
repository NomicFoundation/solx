//!
//! The switch statement case.
//!

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;
use crate::parser::statement::block::Block;
use crate::parser::statement::expression::literal::Literal;

///
/// The Yul switch statement case.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Case {
    /// The location.
    pub location: Location,
    /// The matched constant.
    pub literal: Literal,
    /// The case block.
    pub block: Block,
}

impl Case {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let token = crate::parser::take_or_next(initial, lexer)?;

        let (location, literal) = match token {
            token @ Token {
                lexeme: Lexeme::Literal(_),
                location,
                ..
            } => (location, Literal::parse(lexer, Some(token))?),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{literal}"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };

        let block = Block::parse(lexer, None)?;

        Ok(Self {
            location,
            literal,
            block,
        })
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
    use crate::lexer::Lexer;
    use crate::lexer::token::location::Location;
    use crate::parser::error::Error;
    use crate::parser::statement::object::Object;

    #[test]
    fn error_invalid_token_literal() {
        let input = r#"
object "Test" {
    code {
        {
            return(0, 0)
        }
    }
    object "Test_deployed" {
        code {
            {
                switch 42
                    case x {}
                    default {}
                }
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
                location: Location::new(12, 26),
                expected: vec!["{literal}"],
                found: "x".to_owned(),
            }
            .into())
        );
    }
}
