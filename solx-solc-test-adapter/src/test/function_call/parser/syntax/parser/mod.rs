//!
//! The syntax parser.
//!

pub mod call;
pub mod event;
pub mod gas;
pub mod literal;
pub mod r#type;
pub mod value;

use std::cell::RefCell;
use std::rc::Rc;

use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Token;
use crate::test::function_call::parser::lexical::TokenStream;
use crate::test::function_call::parser::syntax::error::ParsingError;
use crate::test::function_call::parser::syntax::parser::call::Parser as CallParser;
use crate::test::function_call::parser::syntax::parser::gas::Parser as GasParser;
use crate::test::function_call::parser::syntax::tree::call::Call;

///
/// The calls parser.
///
#[derive(Default)]
pub struct Parser {
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a function calls.
    ///
    pub fn parse(mut self, input: &str) -> Result<Vec<Call>, ParsingError> {
        let stream = TokenStream::new(input).wrap();

        let mut calls = Vec::new();
        loop {
            match take_or_next(self.next.take(), stream.clone())? {
                Token {
                    lexeme: Lexeme::Eof,
                    ..
                } => break,
                token @ Token {
                    lexeme: Lexeme::Keyword(Keyword::Gas),
                    ..
                } => {
                    let (_gas, next) = GasParser::default().parse(stream.clone(), Some(token))?;
                    self.next = next;
                }
                token => {
                    let (call, next) = CallParser::default().parse(stream.clone(), Some(token))?;
                    self.next = next;
                    calls.push(call);
                    // Skip gas annotations (parsed and discarded).
                    while let Some(Token {
                        lexeme: Lexeme::Keyword(Keyword::Gas),
                        ..
                    }) = &self.next
                    {
                        let (_gas, next) =
                            GasParser::default().parse(stream.clone(), self.next.take())?;
                        self.next = next;
                    }
                }
            }
        }

        Ok(calls)
    }
}

///
/// Returns the `token` value if it is `Some(_)`, otherwise takes the next token from the `stream`.
///
pub fn take_or_next(
    mut token: Option<Token>,
    stream: Rc<RefCell<TokenStream>>,
) -> Result<Token, ParsingError> {
    match token.take() {
        Some(token) => Ok(token),
        None => Ok(stream.borrow_mut().next()?),
    }
}
