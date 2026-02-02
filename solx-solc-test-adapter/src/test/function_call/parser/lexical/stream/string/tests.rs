//!
//! The lexical string literal parser tests.
//!

use super::Error;
use super::Output;
use super::parse;
use crate::test::function_call::parser::lexical::token::lexeme::literal::string::String;

#[test]
fn ok() {
    let input = r#""some string""#;
    let expected = Ok(Output::new(
        input.len(),
        input.len(),
        input.lines().count() - 1,
        input.len(),
        String::new("some string".to_owned()),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_not_a_string() {
    let input = r#"no double quote here"#;
    let expected = Err(Error::NotAString);
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_unterminated_double_quote() {
    let input = r#""some string"#;
    let expected = Err(Error::UnterminatedDoubleQuote {
        lines: input.lines().count() - 1,
        column: input.len() + 1,
    });
    let result = parse(input);
    assert_eq!(result, expected);
}
