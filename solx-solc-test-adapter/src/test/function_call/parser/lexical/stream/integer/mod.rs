//!
//! The lexical integer literal parser.
//!

#[cfg(test)]
mod tests;

pub mod error;
pub mod output;

use self::error::Error;
use self::output::Output;
use crate::test::function_call::parser::lexical::token::lexeme::literal::integer::Integer;

///
/// The parser state.
///
pub enum State {
    /// The initial state.
    Start,
    /// The `-` has been parsed so far.
    Minus,
    /// The `0` has been parsed so far.
    ZeroOrHexadecimal,
    /// The non-zero decimal character or `-0` or `00` has been parsed so far.
    Decimal,
    /// The `0x` has been parsed so far.
    Hexadecimal,
}

///
/// Parses an integer literal.
///
/// Integer literals can be of two types:
///
/// 1. Decimal
///    Example: '-42'
///
/// 2. Hexadecimal
///    Example: '2a'
///
pub fn parse(input: &str) -> Result<Output, Error> {
    let mut state = State::Start;
    let mut size = 0;

    let mut integer = String::new();
    let mut negative = false;

    loop {
        let character = input.chars().nth(size);
        match state {
            State::Start => match character {
                Some(Integer::CHARACTER_ZERO) => {
                    integer.push(Integer::CHARACTER_ZERO);
                    size += 1;
                    state = State::ZeroOrHexadecimal;
                }
                Some(Integer::CHARACTER_MINUS) => {
                    negative = true;
                    size += 1;
                    state = State::Minus;
                }
                Some(character) => {
                    if !Integer::CHARACTERS_DECIMAL.contains(&character) {
                        return Err(Error::NotAnInteger);
                    }
                    integer.push(character);
                    size += 1;
                    state = State::Decimal;
                }
                None => return Err(Error::NotAnInteger),
            },
            State::Minus => match character {
                Some(character) => {
                    if Integer::CHARACTERS_DECIMAL.contains(&character) {
                        integer.push(character);
                        size += 1;
                        state = State::Decimal;
                    } else {
                        return Err(Error::NotAnInteger);
                    }
                }
                None => return Err(Error::NotAnInteger),
            },
            State::ZeroOrHexadecimal => match character {
                Some(Integer::CHARACTER_INITIAL_HEXADECIMAL) => {
                    size += 1;
                    integer.clear();
                    state = State::Hexadecimal;
                }
                Some(character) => {
                    if Integer::CHARACTERS_DECIMAL.contains(&character) {
                        integer.push(character);
                        size += 1;
                        state = State::Decimal;
                    } else if character.is_ascii_alphabetic() {
                        return Err(Error::ExpectedOneOfDecimalOrX {
                            found: character,
                            offset: size,
                        });
                    } else {
                        return Ok(Output::new(size, Integer::new_decimal(integer, negative)));
                    }
                }
                None => return Ok(Output::new(size, Integer::new_decimal(integer, negative))),
            },
            State::Decimal => match character {
                Some(character) => {
                    if Integer::CHARACTERS_DECIMAL.contains(&character) {
                        integer.push(character);
                        size += 1;
                    } else if character.is_ascii_alphabetic() {
                        return Err(Error::ExpectedOneOfDecimal {
                            found: character,
                            offset: size,
                        });
                    } else {
                        return Ok(Output::new(size, Integer::new_decimal(integer, negative)));
                    }
                }
                None => return Ok(Output::new(size, Integer::new_decimal(integer, negative))),
            },
            State::Hexadecimal => match character {
                Some(character) => {
                    if Integer::CHARACTERS_HEXADECIMAL.contains(&character) {
                        integer.push(character.to_ascii_lowercase());
                        size += 1;
                    } else if character.is_ascii_alphabetic() {
                        return Err(Error::ExpectedOneOfHexadecimal {
                            found: character,
                            offset: size,
                        });
                    } else {
                        if integer.is_empty() {
                            return Err(Error::EmptyHexadecimalBody { offset: size });
                        }
                        return Ok(Output::new(size, Integer::new_hexadecimal(integer)));
                    }
                }
                None => {
                    if integer.is_empty() {
                        return Err(Error::EmptyHexadecimalBody { offset: size });
                    }
                    return Ok(Output::new(size, Integer::new_hexadecimal(integer)));
                }
            },
        }
    }
}
