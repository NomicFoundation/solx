//!
//! The block statement.
//!

pub mod assignment;
pub mod block;
pub mod r#break;
pub mod code;
pub mod r#continue;
pub mod expression;
pub mod for_loop;
pub mod function_definition;
pub mod if_conditional;
pub mod leave;
pub mod object;
pub mod switch;
pub mod variable_declaration;

use std::collections::BTreeSet;

use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::keyword::Keyword;
use crate::yul::lexer::token::location::Location;
use crate::yul::parser::error::Error as ParserError;

use self::assignment::Assignment;
use self::block::Block;
use self::r#break::Break;
use self::code::Code;
use self::r#continue::Continue;
use self::expression::Expression;
use self::for_loop::ForLoop;
use self::function_definition::FunctionDefinition;
use self::if_conditional::IfConditional;
use self::leave::Leave;
use self::object::Object;
use self::switch::Switch;
use self::variable_declaration::VariableDeclaration;

use super::dialect::Dialect;

///
/// The Yul block statement.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(bound = "P: serde::de::DeserializeOwned")]
pub enum Statement<P>
where
    P: Dialect,
{
    /// The object element.
    Object(Object<P>),
    /// The code element.
    Code(Code<P>),
    /// The code block.
    Block(Block<P>),
    /// The expression.
    Expression(Expression),
    /// The `function` statement.
    FunctionDefinition(FunctionDefinition<P>),
    /// The `let` statement.
    VariableDeclaration(VariableDeclaration),
    /// The `:=` existing variables reassignment statement.
    Assignment(Assignment),
    /// The `if` statement.
    IfConditional(IfConditional<P>),
    /// The `switch` statement.
    Switch(Switch<P>),
    /// The `for` statement.
    ForLoop(ForLoop<P>),
    /// The `continue` statement.
    Continue(Continue),
    /// The `break` statement.
    Break(Break),
    /// The `leave` statement.
    Leave(Leave),
}

impl<P> Statement<P>
where
    P: Dialect,
{
    ///
    /// The element parser.
    ///
    pub fn parse(
        lexer: &mut Lexer,
        initial: Option<Token>,
    ) -> Result<(Self, Option<Token>), Error> {
        let token = crate::yul::parser::take_or_next(initial, lexer)?;

        match token {
            Token {
                lexeme: Lexeme::Identifier(ref identifier),
                ..
            } if identifier.inner.as_str() == "object" => Ok((
                Statement::Object(Object::parse(
                    lexer,
                    Some(token),
                    solx_utils::CodeSegment::Deploy,
                )?),
                None,
            )),
            Token {
                lexeme: Lexeme::Identifier(ref identifier),
                ..
            } if identifier.inner.as_str() == "code" => {
                Ok((Statement::Code(Code::parse(lexer, None)?), None))
            }
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::Function),
                ..
            } => Ok((
                Statement::FunctionDefinition(FunctionDefinition::parse(lexer, Some(token))?),
                None,
            )),
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::Let),
                ..
            } => {
                let (statement, next) = VariableDeclaration::parse(lexer, Some(token))?;
                Ok((Statement::VariableDeclaration(statement), next))
            }
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::If),
                ..
            } => Ok((
                Statement::IfConditional(IfConditional::parse(lexer, Some(token))?),
                None,
            )),
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::Switch),
                ..
            } => Ok((Statement::Switch(Switch::parse(lexer, Some(token))?), None)),
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::For),
                ..
            } => Ok((
                Statement::ForLoop(ForLoop::parse(lexer, Some(token))?),
                None,
            )),
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::Continue),
                ..
            } => Ok((
                Statement::Continue(Continue::parse(lexer, Some(token))?),
                None,
            )),
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::Break),
                ..
            } => Ok((Statement::Break(Break::parse(lexer, Some(token))?), None)),
            token @ Token {
                lexeme: Lexeme::Keyword(Keyword::Leave),
                ..
            } => Ok((Statement::Leave(Leave::parse(lexer, Some(token))?), None)),
            token => Err(ParserError::InvalidToken {
                location: token.location,
                expected: vec![
                    "object", "code", "function", "let", "if", "switch", "for", "continue",
                    "break", "leave",
                ],
                found: token.lexeme.to_string(),
            }
            .into()),
        }
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        match self {
            Self::Object(inner) => inner.get_unlinked_libraries(),
            Self::Code(inner) => inner.get_unlinked_libraries(),
            Self::Block(inner) => inner.get_unlinked_libraries(),
            Self::Expression(inner) => inner.get_unlinked_libraries(),
            Self::FunctionDefinition(inner) => inner.get_unlinked_libraries(),
            Self::VariableDeclaration(inner) => inner.get_unlinked_libraries(),
            Self::Assignment(inner) => inner.get_unlinked_libraries(),
            Self::IfConditional(inner) => inner.get_unlinked_libraries(),
            Self::Switch(inner) => inner.get_unlinked_libraries(),
            Self::ForLoop(inner) => inner.get_unlinked_libraries(),
            Self::Continue(_) => BTreeSet::new(),
            Self::Break(_) => BTreeSet::new(),
            Self::Leave(_) => BTreeSet::new(),
        }
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        match self {
            Self::Object(_) => {}
            Self::Code(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::Block(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::Expression(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::FunctionDefinition(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::VariableDeclaration(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::Assignment(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::IfConditional(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::Switch(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::ForLoop(inner) => inner.accumulate_evm_dependencies(dependencies),
            Self::Continue(_) => {}
            Self::Break(_) => {}
            Self::Leave(_) => {}
        }
    }

    ///
    /// Returns the statement location.
    ///
    pub fn location(&self) -> Location {
        match self {
            Self::Object(inner) => inner.location,
            Self::Code(inner) => inner.location,
            Self::Block(inner) => inner.location,
            Self::Expression(inner) => inner.location(),
            Self::FunctionDefinition(inner) => inner.location,
            Self::VariableDeclaration(inner) => inner.location,
            Self::Assignment(inner) => inner.location,
            Self::IfConditional(inner) => inner.location,
            Self::Switch(inner) => inner.location,
            Self::ForLoop(inner) => inner.location,
            Self::Continue(inner) => inner.location,
            Self::Break(inner) => inner.location,
            Self::Leave(inner) => inner.location,
        }
    }
}
