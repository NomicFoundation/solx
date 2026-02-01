//!
//! The lexical integer literal parser tests.
//!

use super::Error;
use super::Output;
use super::parse;
use crate::test::function_call::parser::lexical::token::lexeme::literal::integer::Integer;

#[test]
fn ok_decimal_zero() {
    let input = "0";
    let expected = Ok(Output::new(
        input.len(),
        Integer::new_decimal(input.to_owned(), false),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_decimal() {
    let input = "666";
    let expected = Ok(Output::new(
        input.len(),
        Integer::new_decimal(input.to_owned(), false),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_decimal_negative() {
    let input = "-666";
    let expected = Ok(Output::new(
        input.len(),
        Integer::new_decimal("666".to_owned(), true),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_hexadecimal_lowercase() {
    let input = "0xdead666beef";
    let filtered = "dead666beef";
    let expected = Ok(Output::new(
        input.len(),
        Integer::new_hexadecimal(filtered.to_owned()),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_hexadecimal_uppercase() {
    let input = "0xDEAD666BEEF";
    let filtered = "dead666beef";
    let expected = Ok(Output::new(
        input.len(),
        Integer::new_hexadecimal(filtered.to_owned()),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_hexadecimal_mixed_case() {
    let input = "0xdEaD666bEeF";
    let filtered = "dead666beef";
    let expected = Ok(Output::new(
        input.len(),
        Integer::new_hexadecimal(filtered.to_owned()),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_not_an_integer() {
    let input = "xyz";
    let expected = Err(Error::NotAnInteger);
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_empty_hexadecimal_body() {
    let input = "0x";
    let expected = Err(Error::EmptyHexadecimalBody {
        offset: input.len(),
    });
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_expected_one_of_decimal_or_x() {
    let input = "0f";
    let expected = Err(Error::ExpectedOneOfDecimalOrX {
        found: 'f',
        offset: input.len() - 1,
    });
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_expected_one_of_decimal() {
    let input = "25x";
    let expected = Err(Error::ExpectedOneOfDecimal {
        found: 'x',
        offset: input.len() - 1,
    });
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_expected_one_of_hexadecimal() {
    let input = "0xABCX";
    let expected = Err(Error::ExpectedOneOfHexadecimal {
        found: 'X',
        offset: input.len() - 1,
    });
    let result = parse(input);
    assert_eq!(result, expected);
}
