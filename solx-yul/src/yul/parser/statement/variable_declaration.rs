//!
//! The variable declaration statement.
//!

use std::collections::BTreeSet;

use crate::yul::error::Error;
use crate::yul::lexer::token::lexeme::keyword::Keyword;
use crate::yul::lexer::token::lexeme::symbol::Symbol;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::location::Location;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::Lexer;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::identifier::Identifier;
use crate::yul::parser::statement::expression::function_call::name::Name as FunctionName;
use crate::yul::parser::statement::expression::Expression;

///
/// The Yul variable declaration statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct VariableDeclaration {
    /// The location.
    pub location: Location,
    /// The variable bindings list.
    pub bindings: Vec<Identifier>,
    /// The variable initializing expression.
    pub expression: Option<Expression>,
    /// The solc source code location.
    pub solc_location: Option<solx_utils::DebugInfoSolcLocation>,
}

impl VariableDeclaration {
    ///
    /// The element parser.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Token>,
    ) -> Result<(Self, Option<Token>), Error> {
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
                lexeme: Lexeme::Keyword(Keyword::Let),
                ..
            } => {}
            ref token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["let"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        }

        let (bindings, next) = Identifier::parse_typed_list(lexer, None)?;
        for binding in bindings.iter() {
            match FunctionName::from(binding.inner.as_str()) {
                FunctionName::UserDefined(_) => continue,
                _function_name => {
                    return Err(ParserError::ReservedIdentifier {
                        location: binding.location,
                        identifier: binding.inner.to_owned(),
                    }
                    .into())
                }
            }
        }

        match crate::yul::parser::take_or_next(next, lexer)? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::Assignment),
                ..
            } => {}
            token => {
                return Ok((
                    Self {
                        location,
                        bindings,
                        expression: None,
                        solc_location,
                    },
                    Some(token),
                ))
            }
        }

        let expression = Expression::parse(lexer, None)?;

        Ok((
            Self {
                location,
                bindings,
                expression: Some(expression),
                solc_location,
            },
            None,
        ))
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        self.expression
            .as_ref()
            .map_or_else(BTreeSet::new, |expression| {
                expression.get_unlinked_libraries()
            })
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        if let Some(ref expression) = self.expression {
            expression.accumulate_evm_dependencies(dependencies);
        }
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
    fn unreserved_keyword() {
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
                let object := 42
                let code := 84
                return(0, 0)
            }
        }
    }
}
    "#;

        let mut lexer = Lexer::new(input);
        let result =
            Object::<DefaultDialect>::parse(&mut lexer, None, solx_utils::CodeSegment::Deploy);
        assert!(result.is_ok());
    }

    #[test]
    fn error_reserved_identifier() {
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
                let basefee := 42
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
            Err(Error::ReservedIdentifier {
                location: Location::new(11, 21),
                identifier: "basefee".to_owned()
            }
            .into())
        );
    }
}
