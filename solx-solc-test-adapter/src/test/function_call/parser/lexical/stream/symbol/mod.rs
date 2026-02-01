//!
//! The lexical symbol parser.
//!

#[cfg(test)]
mod tests;

pub mod error;
pub mod output;

use std::str;

use self::error::Error;
use self::output::Output;
use crate::test::function_call::parser::lexical::token::lexeme::symbol::Symbol;

///
/// The parser state.
///
pub enum State {
    /// The initial state.
    Start,
    /// The `-` has been parsed so far.
    Minus,
}

///
/// Parses a symbol.
///
/// Returns the symbol and its size.
///
pub fn parse(input: &str) -> Result<Output, Error> {
    let mut state = State::Start;
    let mut size = 0;

    loop {
        let character = input.chars().nth(size);
        match state {
            State::Start => match character {
                Some('[') => return Ok(Output::new(size + 1, Symbol::BracketSquareLeft)),
                Some(']') => return Ok(Output::new(size + 1, Symbol::BracketSquareRight)),
                Some('(') => return Ok(Output::new(size + 1, Symbol::ParenthesisLeft)),
                Some(')') => return Ok(Output::new(size + 1, Symbol::ParenthesisRight)),
                Some('<') => return Ok(Output::new(size + 1, Symbol::Lesser)),
                Some('>') => return Ok(Output::new(size + 1, Symbol::Greater)),

                Some(':') => return Ok(Output::new(size + 1, Symbol::Colon)),
                Some(',') => return Ok(Output::new(size + 1, Symbol::Comma)),
                Some('~') => return Ok(Output::new(size + 1, Symbol::Tilde)),
                Some('#') => return Ok(Output::new(size + 1, Symbol::Number)),

                Some('-') => {
                    size += 1;
                    state = State::Minus;
                }
                Some(character) => {
                    return Err(Error::InvalidCharacter {
                        found: character,
                        offset: size,
                    });
                }
                None => return Err(Error::UnexpectedEnd),
            },
            State::Minus => {
                return match character {
                    Some('>') => Ok(Output::new(size + 1, Symbol::Arrow)),
                    Some(character) => {
                        return Err(Error::InvalidCharacter {
                            found: character,
                            offset: size,
                        });
                    }
                    None => return Err(Error::UnexpectedEnd),
                };
            }
        }
    }
}
