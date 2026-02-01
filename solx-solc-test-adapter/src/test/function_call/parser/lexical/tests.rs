//!
//! The lexical parser tests.
//!

use super::error::Error;
use super::stream::TokenStream;
use super::token::Token;
use super::token::lexeme::Lexeme;
use super::token::lexeme::identifier::Identifier;
use super::token::lexeme::keyword::Keyword;
use super::token::lexeme::literal::Literal;
use super::token::lexeme::literal::integer::Integer;
use super::token::lexeme::symbol::Symbol;
use super::token::location::Location;

#[test]
fn ok() {
    let input = r#"
# This is the mega ultra test application!
#
f(uint256): 2 -> -2
"#;

    let expected = vec![
        Token {
            lexeme: Lexeme::Identifier(Identifier::new("f".to_owned())),
            location: Location::test(4, 1),
        },
        Token {
            lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
            location: Location::test(4, 2),
        },
        Token {
            lexeme: Lexeme::Keyword(Keyword::new_integer_unsigned(256)),
            location: Location::test(4, 3),
        },
        Token {
            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
            location: Location::test(4, 10),
        },
        Token {
            lexeme: Lexeme::Symbol(Symbol::Colon),
            location: Location::test(4, 11),
        },
        Token {
            lexeme: Lexeme::Literal(Literal::Integer(Integer::new_decimal(
                "2".to_owned(),
                false,
            ))),
            location: Location::test(4, 13),
        },
        Token {
            lexeme: Lexeme::Symbol(Symbol::Arrow),
            location: Location::test(4, 15),
        },
        Token {
            lexeme: Lexeme::Literal(Literal::Integer(Integer::new_decimal("2".to_owned(), true))),
            location: Location::test(4, 18),
        },
    ]
    .into_iter()
    .collect::<Vec<Token>>();

    let mut result = Vec::with_capacity(expected.len());
    let mut stream = TokenStream::test(input);
    loop {
        match stream.next().expect("Always valid") {
            Token {
                lexeme: Lexeme::Eof,
                ..
            } => break,
            token => result.push(token),
        }
    }

    assert_eq!(result, expected);
}

#[test]
fn error_unterminated_double_quote_string() {
    let input = "\"double quote string";

    let expected: Result<Token, Error> = Err(Error::unterminated_double_quote_string(
        Location::test(1, 1),
        Location::test(1, 21),
    ));

    let result = TokenStream::test(input).next();

    assert_eq!(result, expected);
}

#[test]
fn error_expected_one_of_decimal() {
    let input = "42x";

    let expected: Result<Token, Error> = Err(Error::expected_one_of_decimal_integer(
        Location::test(1, 3),
        'x',
    ));

    let result = TokenStream::test(input).next();

    assert_eq!(result, expected);
}

#[test]
fn error_expected_one_of_hexadecimal() {
    let input = "0x42t";

    let expected: Result<Token, Error> = Err(Error::expected_one_of_hexadecimal_integer(
        Location::test(1, 5),
        't',
    ));

    let result = TokenStream::test(input).next();

    assert_eq!(result, expected);
}

#[test]
fn error_invalid_character() {
    let input = "@";

    let expected: Result<Token, Error> = Err(Error::invalid_character(
        Location::test(1, 1),
        input.chars().collect::<Vec<char>>()[0],
    ));

    let result = TokenStream::test(input).next();

    assert_eq!(result, expected);
}

#[test]
fn error_unexpected_end() {
    let input = "0x";

    let expected: Result<Token, Error> = Err(Error::unexpected_end(Location::test(1, 3)));

    let result = TokenStream::test(input).next();

    assert_eq!(result, expected);
}
