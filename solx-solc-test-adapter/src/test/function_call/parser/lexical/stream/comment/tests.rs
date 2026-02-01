//!
//! The lexical comment parser tests.
//!

use super::Error;
use super::Output;
use super::parse;
use crate::test::function_call::parser::lexical::token::lexeme::comment::Comment;

#[test]
fn ok() {
    let input = r#"# mega ultra comment text # "#;
    let expected = Ok(Output::new(
        input.len(),
        input.len(),
        input.lines().count() - 1,
        input.len() + 1,
        Comment::new(" mega ultra comment text ".to_owned()),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn ok_multi_line() {
    let input = r#"# This
is the mega ultra test application!
#
"#;
    let expected = Ok(Output::new(
        input.len(),
        input.len(),
        3,
        1,
        Comment::new(" This\nis the mega ultra test application!\n".to_owned()),
    ));
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_not_a_comment_unterminated_double_quote() {
    let input = r#"not a comment text"#;
    let expected = Err(Error::NotAComment);
    let result = parse(input);
    assert_eq!(result, expected);
}

#[test]
fn error_not_a_comment_no_whitespace_in_the_end() {
    let input = r#"#not a comment text#"#;
    let expected = Err(Error::NotAComment);
    let result = parse(input);
    assert_eq!(result, expected);
}
