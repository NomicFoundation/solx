//!
//! The function call subexpression.
//!

pub mod name;

use std::collections::BTreeSet;

use crate::dependencies::Dependencies;
use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::literal::Literal as LexicalLiteral;
use crate::yul::lexer::token::lexeme::symbol::Symbol;
use crate::yul::lexer::token::location::Location;
use crate::yul::parser::error::Error as ParserError;
use crate::yul::parser::statement::expression::Expression;
use crate::yul::parser::statement::expression::literal::Literal;

use self::name::Name;

///
/// The Yul function call subexpression.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub struct FunctionCall {
    /// The location.
    pub location: Location,
    /// The function name.
    pub name: Name,
    /// The function arguments expression list.
    pub arguments: Vec<Expression>,
}

impl FunctionCall {
    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let token = crate::yul::parser::take_or_next(initial, lexer)?;

        let (location, name) = match token {
            Token {
                lexeme: Lexeme::Identifier(identifier),
                location,
                ..
            } => (location, Name::from(identifier.inner.as_str())),
            token => {
                return Err(ParserError::InvalidToken {
                    location: token.location,
                    expected: vec!["{identifier}"],
                    found: token.lexeme.to_string(),
                }
                .into());
            }
        };

        let mut arguments = Vec::new();
        loop {
            let argument = match lexer.next()? {
                Token {
                    lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                    ..
                } => break,
                token => Expression::parse(lexer, Some(token))?,
            };

            arguments.push(argument);

            match lexer.peek()? {
                Token {
                    lexeme: Lexeme::Symbol(Symbol::Comma),
                    ..
                } => {
                    lexer.next()?;
                    continue;
                }
                Token {
                    lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                    ..
                } => {
                    lexer.next()?;
                    break;
                }
                _ => break,
            }
        }

        Ok(Self {
            location,
            name,
            arguments,
        })
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        let mut libraries = BTreeSet::new();

        if let Name::LinkerSymbol = self.name {
            if let Expression::Literal(Literal {
                inner: LexicalLiteral::String(library_path),
                ..
            }) = self.arguments.first().expect("Always exists")
            {
                libraries.insert(library_path.to_string());
            }
            return libraries;
        }

        for argument in self.arguments.iter() {
            libraries.extend(argument.get_unlinked_libraries());
        }
        libraries
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut Dependencies) {
        match self.name {
            Name::DataSize | Name::DataOffset => {
                if let Expression::Literal(Literal {
                    inner: LexicalLiteral::String(identifier),
                    ..
                }) = self.arguments.first().expect("Always exists")
                {
                    let object_name = identifier.inner.split(".").last().expect("Always exists");
                    let is_runtime_code = dependencies.identifier.as_str()
                        == object_name
                            .strip_suffix("_deployed")
                            .unwrap_or(dependencies.identifier.as_str());
                    dependencies.push(object_name.to_owned(), is_runtime_code);
                }
                return;
            }
            _ => {}
        }

        for argument in self.arguments.iter() {
            argument.accumulate_evm_dependencies(dependencies);
        }
    }
}
