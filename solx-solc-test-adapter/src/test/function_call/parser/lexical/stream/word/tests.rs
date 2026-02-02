//!
//! The lexical word parser tests.
//!

use super::Output;
use super::parse;
use crate::test::function_call::parser::lexical::token::lexeme::Lexeme;
use crate::test::function_call::parser::lexical::token::lexeme::identifier::Identifier;
use crate::test::function_call::parser::lexical::token::lexeme::keyword::Keyword;
use crate::test::function_call::parser::lexical::token::lexeme::literal::Literal;
use crate::test::function_call::parser::lexical::token::lexeme::literal::boolean::Boolean;

#[test]
fn ok_identifier() {
    let input = "xyz";
    let expected = Output::new(
        input.len(),
        Lexeme::Identifier(Identifier::new(input.to_owned())),
    );
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_identifier_below_field_range() {
    let input = "uint0";
    let expected = Output::new(
        input.len(),
        Lexeme::Identifier(Identifier::new(input.to_owned())),
    );
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_identifier_above_field_range() {
    let input = "bytes33";
    let expected = Output::new(
        input.len(),
        Lexeme::Identifier(Identifier::new(input.to_owned())),
    );
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_identifier_invalid_modulo() {
    let input = "uint119";
    let expected = Output::new(
        input.len(),
        Lexeme::Identifier(Identifier::new(input.to_owned())),
    );
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_keyword() {
    let input = "gas";
    let expected = Output::new(input.len(), Lexeme::Keyword(Keyword::Gas));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_keyword_signed_integer_min() {
    let input = "int8";
    let expected = Output::new(input.len(), Lexeme::Keyword(Keyword::new_integer_signed(8)));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_keyword_unsigned_integer_max() {
    let input = "uint256";
    let expected = Output::new(
        input.len(),
        Lexeme::Keyword(Keyword::new_integer_unsigned(solx_utils::BIT_LENGTH_FIELD)),
    );
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_literal_boolean() {
    let input = "true";
    let expected = Output::new(
        input.len(),
        Lexeme::Literal(Literal::Boolean(Boolean::r#true())),
    );
    let result = parse(input);
    assert_eq!(result, expected);
}
