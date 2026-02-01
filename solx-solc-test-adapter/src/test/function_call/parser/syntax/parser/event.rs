//!
//! The event parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

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
use crate::test::function_call::parser::syntax::parser::literal::Parser as LiteralParser;
use crate::test::function_call::parser::syntax::parser::r#type::Parser as TypeParser;
use crate::test::function_call::parser::syntax::tree::event::Event;
use crate::test::function_call::parser::syntax::tree::event::builder::Builder as EventBuilder;
use crate::test::function_call::parser::syntax::tree::event::literal::EventLiteral;
use crate::test::function_call::parser::syntax::tree::identifier::Identifier;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    Start,
    /// The `~` has been parsed so far.
    Emit,
    /// The `~ emit` has been parsed so far.
    IdentifierOrLesser,
    /// The `~ emit {identifier}` has been parsed so far.
    ParenthesisLeft,
    /// The `~ emit {identifier}(` has been parsed so far.
    TypeOrParenthesisRight,
    /// The `~ emit {identifier}({type}` has been parsed so far.
    CommaOrParenthesisRight,
    /// The `~ emit {identifier}({type},` has been parsed so far.
    Type,
    /// The `~ emit <` has been parsed so far.
    Anonymous,
    /// The `~ emit <anonymous` has been parsed so far.
    Greater,
    /// The `~ emit {identifier}({types})` or `~emit <anonymous>` has been parsed so far.
    ColonFromOrEnd,
    /// The `~ emit {identifier}({types}) from` or `~emit <anonymous> from` has been parsed so far.
    Address,
    /// The `~ emit {identifier}({types}) from {address}` or `~emit <anonymous> from {address}` has been parsed so far.
    ColonOrEnd,
    /// The `~ emit {identifier}({types}):` or `~emit <anonymous>:` has been parsed so far.
    LiteralOrEnd,
    /// The `...:#` has been parsed so far.
    IndexedLiteral,
    /// The `...:{literal}` has been parsed so far.
    CommaOrEnd,
    /// The `...:{literal},` has been parsed so far.
    Literal,
}

///
/// The event parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: EventBuilder,
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a even.
    ///
    /// '
    /// ~ emit Verified(string): 0x20, 0x16, "Successfully verified."
    /// ~ emit <anonymous>
    /// '
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        initial: Option<Token>,
    ) -> Result<(Event, Option<Token>), ParsingError> {
        self.next = initial;

        loop {
            match self.state {
                State::Start => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.state = State::Emit;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["~"], lexeme).into());
                    }
                },
                State::Emit => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Emit),
                        ..
                    } => {
                        self.state = State::IdentifierOrLesser;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["emit"], lexeme).into());
                    }
                },
                State::IdentifierOrLesser => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Lesser),
                            ..
                        } => {
                            self.state = State::Anonymous;
                        }
                        Token {
                            lexeme: Lexeme::Identifier(identifier),
                            location,
                        } => {
                            self.builder
                                .set_identifier(Identifier::new(location, identifier.inner));
                            self.state = State::ParenthesisLeft;
                        }
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(
                                location,
                                vec!["<", "{identifier}"],
                                lexeme,
                            )
                            .into());
                        }
                    }
                }
                State::ParenthesisLeft => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                            ..
                        } => {
                            self.state = State::TypeOrParenthesisRight;
                        }
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(location, vec!["("], lexeme).into());
                        }
                    }
                }
                State::TypeOrParenthesisRight => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => {
                            self.state = State::ColonFromOrEnd;
                        }
                        token => {
                            let (r#type, next) =
                                TypeParser::default().parse(stream.clone(), Some(token))?;
                            self.next = next;
                            self.builder.push_type(r#type);
                            self.state = State::CommaOrParenthesisRight;
                        }
                    }
                }
                State::CommaOrParenthesisRight => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Comma),
                            ..
                        } => {
                            self.state = State::Type;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => {
                            self.state = State::ColonFromOrEnd;
                        }
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(location, vec![",", ")"], lexeme).into());
                        }
                    }
                }
                State::Type => {
                    let (r#type, next) =
                        TypeParser::default().parse(stream.clone(), self.next.take())?;
                    self.next = next;
                    self.builder.push_type(r#type);
                    self.state = State::CommaOrParenthesisRight;
                }
                State::Anonymous => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Anonymous),
                        ..
                    } => {
                        self.state = State::Greater;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["anonymous"], lexeme).into());
                    }
                },
                State::Greater => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Greater),
                        ..
                    } => {
                        self.state = State::ColonFromOrEnd;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec![">"], lexeme).into());
                    }
                },
                State::ColonFromOrEnd => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Colon),
                            ..
                        } => {
                            self.builder.set_is_expected();
                            self.state = State::LiteralOrEnd;
                        }
                        Token {
                            lexeme: Lexeme::Keyword(Keyword::From),
                            ..
                        } => {
                            self.state = State::Address;
                        }
                        token => return Ok((self.builder.finish(), Some(token))),
                    }
                }
                State::Address => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme:
                            Lexeme::Literal(LexicalLiteral::Integer(
                                LexicalIntegerLiteral::Hexadecimal(hexadecimal_integer),
                            )),
                        ..
                    } => {
                        self.builder.set_address(hexadecimal_integer);
                        self.state = State::ColonOrEnd;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec!["{hexadecimal integer literal}"],
                            lexeme,
                        )
                        .into());
                    }
                },
                State::ColonOrEnd => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Colon),
                            ..
                        } => {
                            self.builder.set_is_expected();
                            self.state = State::LiteralOrEnd;
                        }
                        token => return Ok((self.builder.finish(), Some(token))),
                    }
                }
                State::LiteralOrEnd => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Number),
                            ..
                        } => {
                            self.state = State::IndexedLiteral;
                        }
                        token => {
                            if let Ok((literal, next)) =
                                LiteralParser::default().parse(stream.clone(), Some(token))
                            {
                                self.builder
                                    .push_expected(EventLiteral::new(literal, false));
                                self.next = next;
                                self.state = State::CommaOrEnd;
                            } else {
                                return Ok((self.builder.finish(), self.next));
                            }
                        }
                    }
                }
                State::IndexedLiteral => {
                    let (r#literal, next) =
                        LiteralParser::default().parse(stream.clone(), self.next.take())?;
                    self.next = next;
                    self.builder
                        .push_expected(EventLiteral::new(r#literal, true));
                    self.state = State::CommaOrEnd;
                }
                State::CommaOrEnd => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Comma),
                            ..
                        } => {
                            self.state = State::Literal;
                        }
                        token => return Ok((self.builder.finish(), Some(token))),
                    }
                }
                State::Literal => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Number),
                        ..
                    } => {
                        self.state = State::IndexedLiteral;
                    }
                    token => {
                        let (r#literal, next) =
                            LiteralParser::default().parse(stream.clone(), Some(token))?;
                        self.next = next;
                        self.builder
                            .push_expected(EventLiteral::new(r#literal, false));
                        self.state = State::CommaOrEnd;
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test::function_call::parser::lexical::token::lexeme::identifier::Identifier as LexicalIdentifier;
    use crate::test::function_call::parser::syntax::tree::literal::integer::Literal as IntegerLiteral;

    use crate::test::function_call::parser::lexical::Lexeme;

    use crate::test::function_call::parser::lexical::Location;

    use crate::test::function_call::parser::lexical::IntegerLiteral as LexicalIntegerLiteral;
    use crate::test::function_call::parser::lexical::Token;
    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::Identifier;
    use crate::test::function_call::parser::syntax::Type;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;
    use crate::test::function_call::parser::syntax::tree::event::Event;
    use crate::test::function_call::parser::syntax::tree::event::literal::EventLiteral;
    use crate::test::function_call::parser::syntax::tree::event::variant::Variant;
    use crate::test::function_call::parser::syntax::tree::literal::Literal;
    use crate::test::function_call::parser::syntax::tree::literal::alignment::Alignment;
    use crate::test::function_call::parser::syntax::tree::r#type::variant::Variant as TypeVariant;

    #[test]
    fn ok() {
        let input = r#"~ emit Verified(string) from 0xf0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0: #0x20, -10"#;
        let expected = Ok((
            Event::new(
                Location::test(1, 1),
                Variant::Signature {
                    identifier: Identifier::new(Location::test(1, 8), "Verified".to_owned()),
                    types: vec![Type::new(Location::test(1, 17), TypeVariant::String)],
                },
                Some("f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0".to_owned()),
                Some(vec![
                    EventLiteral::new(
                        Literal::Integer(IntegerLiteral::new(
                            Location::test(1, 75),
                            LexicalIntegerLiteral::Hexadecimal("20".to_owned()),
                            Alignment::Default,
                        )),
                        true,
                    ),
                    EventLiteral::new(
                        Literal::Integer(IntegerLiteral::new(
                            Location::test(1, 81),
                            LexicalIntegerLiteral::new_decimal("10".to_owned(), true),
                            Alignment::Default,
                        )),
                        false,
                    ),
                ]),
            ),
            Some(Token::new(Lexeme::Eof, Location::test(1, 84))),
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_anonymous() {
        let input = r#"~ emit <anonymous>"#;
        let expected = Ok((
            Event::new(Location::test(1, 1), Variant::Anonymous, None, None),
            Some(Token::new(Lexeme::Eof, Location::test(1, 19))),
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn expected_emit() {
        let input = r#"~ e_mit <anonymous>"#;
        let expected = Err(ParsingError::Syntax(SyntaxError::new(
            Location::test(1, 3),
            vec!["emit"],
            Lexeme::Identifier(LexicalIdentifier::new("e_mit".to_owned())),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
