//!
//! The lexical word parser.
//!

#[cfg(test)]
mod tests;

pub mod output;

use std::str::FromStr;

use crate::test::function_call::parser::lexical::token::lexeme::Lexeme;
use crate::test::function_call::parser::lexical::token::lexeme::identifier::Error as IdentifierError;
use crate::test::function_call::parser::lexical::token::lexeme::identifier::Identifier;
use crate::test::function_call::parser::lexical::token::lexeme::literal::Literal;
use crate::test::function_call::parser::lexical::token::lexeme::literal::boolean::Boolean;

use self::output::Output;

///
/// The parser state.
///
pub enum State {
    /// The initial state.
    Start,
    /// The first character has been parsed so far.
    Continue,
}

///
/// Parses a word. The word can result into several token types:
///
/// 1. An identifier
///    Example: 'value'
///    Any valid identifier which is not a keyword.
///
/// 2. A boolean literal
///    Example: 'true'
///    The literal is also a keyword, but is was decided to treat literals as a separate token type.
///
/// 3. A keyword
///    Example: 'emit'
///    Any keyword which is not a boolean literal.
///
pub fn parse(input: &str) -> Output {
    let mut state = State::Start;
    let mut size = 0;

    while let Some(character) = input.chars().nth(size) {
        match state {
            State::Start => {
                if !Identifier::can_start_with(character) {
                    break;
                }
                state = State::Continue;
            }
            State::Continue => {
                if !Identifier::can_contain_after_start(character) {
                    break;
                }
            }
        }

        size += 1;
    }

    let lexeme = match Identifier::from_str(&input[..size]) {
        Ok(identifier) => Lexeme::Identifier(identifier),
        Err(IdentifierError::IsKeyword(keyword)) => match Boolean::try_from(keyword) {
            Ok(boolean) => Lexeme::Literal(Literal::Boolean(boolean)),
            Err(keyword) => Lexeme::Keyword(keyword),
        },
    };
    Output::new(size, lexeme)
}
