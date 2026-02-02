//!
//! The lexical hex literal parser tests.
//!

use super::Error;
use super::Output;
use super::parse;
use crate::test::function_call::parser::lexical::token::lexeme::literal::hex::Hex;

#[test]
fn ok() {
    let input = r#"hex"1234abcd""#;
    let expected = Ok(Output::new(input.len(), Hex::new("1234abcd".to_owned())));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_not_a_hex() {
    let input = r#"hex "abc""#;
    let expected = Err(Error::NotAHex);
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_unterminated_double_quote() {
    let input = r#"hex"1234"#;
    let expected = Err(Error::UnterminatedDoubleQuote {
        offset: input.len(),
    });
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_expected_one_of_hexadecimal() {
    let input = r#"hex"12345g""#;
    let expected = Err(Error::ExpectedOneOfHexadecimal {
        found: 'g',
        offset: input.len() - 1,
    });
    let result = parse(input);
    assert_eq!(result, expected);
}
