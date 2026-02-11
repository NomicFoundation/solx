//!
//! The expression statement.
//!

pub mod function_call;
pub mod literal;

use solx_codegen_evm::IContext;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::symbol::Symbol;
use crate::lexer::token::location::Location;
use crate::parser::error::Error as ParserError;
use crate::parser::identifier::Identifier;

use self::function_call::FunctionCall;
use self::literal::Literal;

///
/// The Yul expression statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum Expression {
    /// The function call subexpression.
    FunctionCall(FunctionCall),
    /// The identifier operand.
    Identifier(Identifier),
    /// The literal operand.
    Literal(Literal),
}

impl Expression {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let token = crate::parser::take_or_next(initial, lexer)?;

        let (location, identifier) = match token {
            Token {
                lexeme: Lexeme::Literal(_),
                ..
            } => return Ok(Self::Literal(Literal::parse(lexer, Some(token))?)),
            Token {
                location,
                lexeme: Lexeme::Identifier(identifier),
                ..
            } => (location, identifier),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{literal}", "{identifier}"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };
        let length = identifier.inner.len();

        match lexer.peek()? {
            Token {
                lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                ..
            } => {
                lexer.next()?;
                Ok(Self::FunctionCall(FunctionCall::parse(
                    lexer,
                    Some(Token::new(location, Lexeme::Identifier(identifier), length)),
                )?))
            }
            _ => Ok(Self::Identifier(Identifier::new(
                location,
                identifier.inner,
            ))),
        }
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        match self {
            Self::FunctionCall(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::Identifier(_) => {}
            Self::Literal(_) => {}
        }
    }

    ///
    /// Returns the statement location.
    ///
    pub fn location(&self) -> Location {
        match self {
            Self::FunctionCall(inner) => inner.location,
            Self::Identifier(inner) => inner.location,
            Self::Literal(inner) => inner.location,
        }
    }

    ///
    /// Converts the expression into an LLVM value.
    ///
    pub fn into_llvm<'ctx>(
        self,
        context: &mut solx_codegen_evm::Context<'ctx>,
    ) -> anyhow::Result<Option<solx_codegen_evm::Value<'ctx>>> {
        match self {
            Expression::Literal(literal) => literal
                .clone()
                .into_llvm(context)
                .map_err(|error| {
                    anyhow::anyhow!(
                        "{} Invalid literal `{}`: {error}",
                        literal.location,
                        literal.inner,
                    )
                })
                .map(Some),
            Expression::Identifier(identifier) => {
                let pointer = context
                    .current_function()
                    .borrow()
                    .get_stack_pointer(identifier.inner.as_str())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "{} Undeclared variable `{}`",
                            identifier.location,
                            identifier.inner,
                        )
                    })?;

                let value = context.build_load(pointer, identifier.inner.as_str())?;
                Ok(Some(value.into()))
            }
            Expression::FunctionCall(call) => {
                Ok(call.into_llvm(context)?.map(solx_codegen_evm::Value::new))
            }
        }
    }
}
