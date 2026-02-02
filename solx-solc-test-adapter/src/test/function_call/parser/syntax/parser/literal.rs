//!
//! The literal parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Symbol;
use crate::test::function_call::parser::lexical::Token;
use crate::test::function_call::parser::lexical::TokenStream;
use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
use crate::test::function_call::parser::syntax::error::ParsingError;
use crate::test::function_call::parser::syntax::parser;
use crate::test::function_call::parser::syntax::tree::literal::Literal;
use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;
use crate::test::function_call::parser::syntax::tree::literal::builder::Builder as LiteralBuilder;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    Start,
    /// The `{alignment}` has been parsed so far.
    Alignment,
    /// The `{alignment}(` has been parsed so far.
    ParenthesisLeft,
    /// The `{alignment}([-]{literal}` has been parsed so far.
    Literal,
}

///
/// The literal parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: LiteralBuilder,
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a literal.
    ///
    /// '
    /// -1234
    /// left(0x12)
    /// hex"1234"
    /// "abc"
    /// '
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        initial: Option<Token>,
    ) -> Result<(Literal, Option<Token>), ParsingError> {
        self.next = initial;

        loop {
            match self.state {
                State::Start => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Literal(literal),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.builder.set_literal(literal);
                        return Ok((self.builder.finish(), None));
                    }
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Left),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.builder.set_alignment(Alignment::left());
                        self.state = State::Alignment;
                    }
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Right),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.builder.set_alignment(Alignment::right());
                        self.state = State::Alignment;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec!["{literal}", "left", "right"],
                            lexeme,
                        )
                        .into());
                    }
                },
                State::Alignment => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                        ..
                    } => {
                        self.state = State::ParenthesisLeft;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["("], lexeme).into());
                    }
                },
                State::ParenthesisLeft => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Literal(literal),
                            ..
                        } => {
                            self.builder.set_literal(literal);
                            self.state = State::Literal;
                        }
                        Token { lexeme, location } => {
                            return Err(
                                SyntaxError::new(location, vec!["{literal}"], lexeme).into()
                            );
                        }
                    }
                }
                State::Literal => {
                    return match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => Ok((self.builder.finish(), None)),
                        Token { lexeme, location } => {
                            Err(SyntaxError::new(location, vec![")"], lexeme).into())
                        }
                    };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::function_call::parser::lexical::HexLiteral as LexicalHexLiteral;
    use crate::test::function_call::parser::lexical::IntegerLiteral as LexicalIntegerLiteral;
    use crate::test::function_call::parser::lexical::Lexeme;
    use crate::test::function_call::parser::lexical::Location;
    use crate::test::function_call::parser::lexical::Symbol;
    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;
    use crate::test::function_call::parser::syntax::tree::literal::Literal;
    use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;
    use crate::test::function_call::parser::syntax::tree::literal::hex::Literal as HexLiteral;
    use crate::test::function_call::parser::syntax::tree::literal::integer::Literal as IntegerLiteral;

    #[test]
    fn ok_integer() {
        let input = r#"-1234"#;

        let expected = Ok((
            Literal::Integer(IntegerLiteral::new(
                Location::test(1, 1),
                LexicalIntegerLiteral::new_decimal("1234".to_owned(), true),
                Alignment::Default,
            )),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_hex() {
        let input = r#"right(hex"1234")"#;

        let expected = Ok((
            Literal::Hex(HexLiteral::new(
                Location::test(1, 1),
                LexicalHexLiteral::new("1234".to_owned()),
                Alignment::Right,
            )),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_parenthesis_left() {
        let input = r#"left~"#;

        let expected = Err(ParsingError::from(SyntaxError::new(
            Location::test(1, 5),
            vec!["("],
            Lexeme::Symbol(Symbol::Tilde),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
