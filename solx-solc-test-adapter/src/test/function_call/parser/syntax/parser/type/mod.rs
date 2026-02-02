//!
//! The type parser.
//!

pub mod tuple;

use std::cell::RefCell;
use std::rc::Rc;

use self::tuple::Parser as TupleParser;
use crate::test::function_call::parser::lexical::IntegerLiteral as LexicalIntegerLiteral;
use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Literal as LexicalLiteral;
use crate::test::function_call::parser::lexical::Symbol;
use crate::test::function_call::parser::lexical::Token;
use crate::test::function_call::parser::lexical::TokenStream;
use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
use crate::test::function_call::parser::syntax::error::ParsingError;
use crate::test::function_call::parser::syntax::parser;
use crate::test::function_call::parser::syntax::tree::r#type::Type;
use crate::test::function_call::parser::syntax::tree::r#type::builder::Builder as TypeBuilder;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    Start,
    /// The `{type}` has been parsed so far.
    BracketSquareLeftOrEnd,
    /// The `{type}[` has been parsed so far.
    SizeOrBracketSquareRight,
    /// The `{type}[{size}` has been parsed so far.
    BracketSquareRight,
}

///
/// The type parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: TypeBuilder,
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a type.
    ///
    /// 'bool'
    /// 'uint8[16]'
    /// '(uint8, uint256, bool)'
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        initial: Option<Token>,
    ) -> Result<(Type, Option<Token>), ParsingError> {
        self.next = initial;
        let mut tuple = None;

        loop {
            match self.state {
                State::Start => match parser::take_or_next(self.next.take(), stream.clone())? {
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                        ..
                    } => {
                        let (value, next) =
                            TupleParser::default().parse(stream.clone(), Some(token))?;
                        tuple = Some(value);
                        self.next = next;
                        self.state = State::BracketSquareLeftOrEnd;
                    }
                    Token {
                        lexeme:
                            Lexeme::Keyword(
                                keyword @ Keyword::Bool
                                | keyword @ Keyword::String
                                | keyword @ Keyword::Address
                                | keyword @ Keyword::Function
                                | keyword @ Keyword::IntegerUnsigned { .. }
                                | keyword @ Keyword::IntegerSigned { .. }
                                | keyword @ Keyword::Bytes { .. },
                            ),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.builder.set_keyword(keyword);
                        self.state = State::BracketSquareLeftOrEnd;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec![
                                "(", "bool", "string", "address", "function", "uint{n}", "int{n}",
                                "bytes", "bytes{n}",
                            ],
                            lexeme,
                        )
                        .into());
                    }
                },
                State::BracketSquareLeftOrEnd => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketSquareLeft),
                            ..
                        } => {
                            let base_type = if let Some(tuple) = tuple.take() {
                                tuple
                            } else {
                                self.builder.finish()
                            };
                            self.builder = TypeBuilder::default();
                            self.builder.set_location(base_type.location);
                            self.builder.set_array_type(base_type);
                            self.state = State::SizeOrBracketSquareRight;
                        }
                        token => {
                            return Ok((
                                tuple.unwrap_or_else(|| self.builder.finish()),
                                Some(token),
                            ));
                        }
                    }
                }
                State::SizeOrBracketSquareRight => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme:
                                Lexeme::Literal(LexicalLiteral::Integer(
                                    LexicalIntegerLiteral::Decimal {
                                        inner: decimal_integer,
                                        negative: false,
                                    },
                                )),
                            ..
                        } => {
                            self.builder.set_array_size(decimal_integer);
                            self.state = State::BracketSquareRight;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketSquareRight),
                            ..
                        } => {
                            self.state = State::BracketSquareLeftOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(
                                location,
                                vec!["{positive decimal integer literal}", "]"],
                                lexeme,
                            )
                            .into());
                        }
                    }
                }
                State::BracketSquareRight => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::BracketSquareRight),
                            ..
                        } => {
                            self.state = State::BracketSquareLeftOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(location, vec!["]"], lexeme).into());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test::function_call::parser::lexical::IntegerLiteral as LexicalIntegerLiteral;

    use crate::test::function_call::parser::lexical::Lexeme;
    use crate::test::function_call::parser::lexical::Literal as LexicalLiteral;
    use crate::test::function_call::parser::lexical::Location;

    use crate::test::function_call::parser::lexical::Token;
    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;

    use crate::test::function_call::parser::syntax::tree::r#type::Type;
    use crate::test::function_call::parser::syntax::tree::r#type::variant::Variant as TypeVariant;

    #[test]
    fn ok_integer() {
        let input = r#"uint232"#;

        let expected = Ok((
            Type::new(Location::test(1, 1), TypeVariant::integer_unsigned(232)),
            Some(Token::new(Lexeme::Eof, Location::test(1, 8))),
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_two_dimensional_array() {
        let input = r#"bytes[5][]"#;

        let expected = Ok((
            Type::new(
                Location::test(1, 1),
                TypeVariant::array(
                    Type::new(
                        Location::test(1, 1),
                        TypeVariant::array(
                            Type::new(Location::test(1, 1), TypeVariant::bytes(None)),
                            Some("5".to_owned()),
                        ),
                    ),
                    None,
                ),
            ),
            Some(Token::new(Lexeme::Eof, Location::test(1, 11))),
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_type() {
        let input = r#"42"#;

        let expected = Err(ParsingError::Syntax(SyntaxError::new(
            Location::test(1, 1),
            vec![
                "(", "bool", "string", "address", "function", "uint{n}", "int{n}", "bytes",
                "bytes{n}",
            ],
            Lexeme::Literal(LexicalLiteral::Integer(LexicalIntegerLiteral::new_decimal(
                "42".to_owned(),
                false,
            ))),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
