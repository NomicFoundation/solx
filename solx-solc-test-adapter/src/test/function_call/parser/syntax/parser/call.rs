//!
//! The call parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Literal as LexicalLiteral;
use crate::test::function_call::parser::lexical::Symbol;
use crate::test::function_call::parser::lexical::Token;
use crate::test::function_call::parser::lexical::TokenStream;
use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
use crate::test::function_call::parser::syntax::error::ParsingError;
use crate::test::function_call::parser::syntax::parser;
use crate::test::function_call::parser::syntax::parser::event::Parser as EventParser;
use crate::test::function_call::parser::syntax::parser::literal::Parser as LiteralParser;
use crate::test::function_call::parser::syntax::parser::r#type::Parser as TypeParser;
use crate::test::function_call::parser::syntax::parser::value::Parser as ValueParser;
use crate::test::function_call::parser::syntax::tree::call::Call;
use crate::test::function_call::parser::syntax::tree::call::builder::Builder as CallBuilder;
use crate::test::function_call::parser::syntax::tree::identifier::Identifier;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    Start,
    /// The `library` has been parsed so far.
    Library,
    /// The `library:` has been parsed so far.
    ColonLibrary,
    /// The `library: "{source file name}"` has been parsed so far.
    Source,
    /// The `library: "{source file name}":` has been parsed so far.
    ColonSource,
    /// The `{function name}` has been parsed so far.
    FunctionName,
    /// The `{function name}(` has been parsed so far.
    ParenthesisLeft,
    /// The `{function name}({type}` has been parsed so far.
    Type,
    /// The `{function name}({type},` has been parsed so far.
    CommaType,
    /// The `{function name}{types}` has been parsed so far.
    Signature,
    /// The `{signature}{value option}` has been parsed so far.
    Value,
    /// The `{signature}[value option]:` has been parsed so far.
    Colon,
    /// The `{signature}[value option]:{literal}` has been parsed so far.
    Input,
    /// The `{signature}[value option]:{literal},` has been parsed so far.
    CommaInput,
    /// The `...->` has been parsed so far.
    Arrow,
    /// The `...->{literal}` or `...->{failure}` has been parsed so far.
    Expected,
    /// The `...->{literal},` or `...->{failure},` has been parsed so far.
    CommaExpected,
    /// The `...{event}: ` has been parsed so far.
    Event,
}

///
/// The call parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: CallBuilder,
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
    ) -> Result<(Call, Option<Token>), ParsingError> {
        self.next = initial;

        loop {
            match self.state {
                State::Start => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Library),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.state = State::Library;
                    }
                    Token {
                        lexeme: Lexeme::Identifier(identifier),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.builder
                            .set_call(Identifier::new(location, identifier.inner));
                        self.state = State::FunctionName;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.builder.set_is_types();
                        self.state = State::ParenthesisLeft;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec!["library", "{identifier}", "("],
                            lexeme,
                        )
                        .into());
                    }
                },
                State::Library => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    } => {
                        self.state = State::ColonLibrary;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec![":"], lexeme).into());
                    }
                },
                State::ColonLibrary => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Identifier(identifier),
                            location,
                        } => {
                            self.builder
                                .set_library(Identifier::new(location, identifier.inner));
                            return Ok((
                                self.builder
                                    .finish()
                                    .expect("Unreachable as long as the parser works correctly"),
                                None,
                            ));
                        }
                        Token {
                            lexeme: Lexeme::Literal(LexicalLiteral::String(string)),
                            ..
                        } => {
                            self.builder.set_library_source(string.inner);
                            self.state = State::Source;
                        }
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(
                                location,
                                vec!["{identifier}", "{string literal}"],
                                lexeme,
                            )
                            .into());
                        }
                    }
                }
                State::Source => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    } => {
                        self.state = State::ColonSource;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec![":"], lexeme).into());
                    }
                },
                State::ColonSource => {
                    return match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Identifier(identifier),
                            location,
                        } => {
                            self.builder
                                .set_library(Identifier::new(location, identifier.inner));
                            Ok((
                                self.builder
                                    .finish()
                                    .expect("Unreachable as long as the parser works correctly"),
                                None,
                            ))
                        }
                        Token { lexeme, location } => {
                            Err(SyntaxError::new(location, vec!["{identifier}"], lexeme).into())
                        }
                    };
                }
                State::FunctionName => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Comma),
                            ..
                        } => {
                            let (value, next) =
                                ValueParser::default().parse(stream.clone(), None)?;
                            self.builder.set_value(value);
                            self.next = next;
                            self.state = State::Value;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Colon),
                            ..
                        } => {
                            self.builder.set_is_input();
                            self.state = State::Colon;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Arrow),
                            ..
                        } => {
                            self.builder.set_is_expected();
                            self.state = State::Arrow;
                        }
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                            ..
                        } => {
                            self.builder.set_is_types();
                            self.state = State::ParenthesisLeft;
                        }
                        token @ Token {
                            lexeme: Lexeme::Symbol(Symbol::Tilde),
                            ..
                        } => {
                            let (event, next) =
                                EventParser::default().parse(stream.clone(), Some(token))?;
                            self.builder.push_event(event);
                            self.next = next;
                            self.state = State::Event;
                        }
                        token => {
                            return Ok((
                                self.builder
                                    .finish()
                                    .expect("Unreachable as long as the parser works correctly"),
                                Some(token),
                            ));
                        }
                    }
                }
                State::ParenthesisLeft => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => {
                            self.state = State::Signature;
                        }
                        token => {
                            let (r#type, next) =
                                TypeParser::default().parse(stream.clone(), Some(token))?;
                            self.builder.push_types(r#type);
                            self.next = next;
                            self.state = State::Type;
                        }
                    }
                }
                State::Type => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                        ..
                    } => {
                        self.state = State::Signature;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Comma),
                        ..
                    } => {
                        self.state = State::CommaType;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec![")", ","], lexeme).into());
                    }
                },
                State::CommaType => {
                    let (r#type, next) = TypeParser::default().parse(stream.clone(), self.next)?;
                    self.builder.push_types(r#type);
                    self.next = next;
                    self.state = State::Type;
                }
                State::Signature => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Comma),
                        ..
                    } => {
                        let (value, next) = ValueParser::default().parse(stream.clone(), None)?;
                        self.builder.set_value(value);
                        self.next = next;
                        self.state = State::Value;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    } => {
                        self.builder.set_is_input();
                        self.state = State::Colon;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Arrow),
                        ..
                    } => {
                        self.builder.set_is_expected();
                        self.state = State::Arrow;
                    }
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
                State::Value => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    } => {
                        self.builder.set_is_input();
                        self.state = State::Colon;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Arrow),
                        ..
                    } => {
                        self.builder.set_is_expected();
                        self.state = State::Arrow;
                    }
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
                State::Colon => match parser::take_or_next(self.next.take(), stream.clone())? {
                    token @ Token {
                        lexeme:
                            Lexeme::Keyword(Keyword::Right)
                            | Lexeme::Keyword(Keyword::Left)
                            | Lexeme::Literal(_),
                        ..
                    } => {
                        let (input, next) =
                            LiteralParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_input(input);
                        self.next = next;
                        self.state = State::Input;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Arrow),
                        ..
                    } => {
                        self.builder.set_is_expected();
                        self.state = State::Arrow;
                    }
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
                State::Input => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Comma),
                        ..
                    } => {
                        self.state = State::CommaInput;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Arrow),
                        ..
                    } => {
                        self.builder.set_is_expected();
                        self.state = State::Arrow;
                    }
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
                State::CommaInput => {
                    let (input, next) =
                        LiteralParser::default().parse(stream.clone(), self.next)?;
                    self.builder.push_input(input);
                    self.next = next;
                    self.state = State::Input;
                }
                State::Arrow => match parser::take_or_next(self.next.take(), stream.clone())? {
                    token @ Token {
                        lexeme:
                            Lexeme::Keyword(Keyword::Right)
                            | Lexeme::Keyword(Keyword::Left)
                            | Lexeme::Literal(_),
                        ..
                    } => {
                        let (expected, next) =
                            LiteralParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_expected(expected);
                        self.next = next;
                        self.state = State::Expected;
                    }
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Failure),
                        ..
                    } => {
                        self.builder.failure = true;
                        self.state = State::Expected;
                    }
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
                State::Expected => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Comma),
                        ..
                    } => {
                        self.state = State::CommaExpected;
                    }
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
                State::CommaExpected => {
                    let (expected, next) =
                        LiteralParser::default().parse(stream.clone(), self.next)?;
                    self.builder.push_expected(expected);
                    self.next = next;
                    self.state = State::Expected;
                }
                State::Event => match parser::take_or_next(self.next.take(), stream.clone())? {
                    token @ Token {
                        lexeme: Lexeme::Symbol(Symbol::Tilde),
                        ..
                    } => {
                        let (event, next) =
                            EventParser::default().parse(stream.clone(), Some(token))?;
                        self.builder.push_event(event);
                        self.next = next;
                        self.state = State::Event;
                    }
                    token => {
                        return Ok((
                            self.builder
                                .finish()
                                .expect("Unreachable as long as the parser works correctly"),
                            Some(token),
                        ));
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test::function_call::parser::lexical::Keyword;
    use crate::test::function_call::parser::lexical::Lexeme;
    use crate::test::function_call::parser::lexical::Location;
    use crate::test::function_call::parser::lexical::Symbol;
    use crate::test::function_call::parser::lexical::Token;
    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;
    use crate::test::function_call::parser::syntax::tree::call::Call;
    use crate::test::function_call::parser::syntax::tree::call::variant::Variant as CallVariant;

    use crate::test::function_call::parser::syntax::Identifier;
    use crate::test::function_call::parser::syntax::tree::event::Event;
    use crate::test::function_call::parser::syntax::tree::event::variant::Variant as EventVariant;
    use crate::test::function_call::parser::syntax::tree::r#type::variant::Variant as TypeVariant;
    use crate::test::function_call::parser::syntax::tree::value::Value;
    use crate::test::function_call::parser::syntax::tree::value::unit::Unit;

    use crate::test::function_call::parser::syntax::Type;

    #[test]
    fn ok_library() {
        let input = r#"library: L"#;
        let expected = Ok((
            Call::new(
                Location::test(1, 1),
                CallVariant::library(Identifier::new(Location::test(1, 10), "L".to_owned()), None),
            ),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_library_with_source() {
        let input = r#"library: "a.sol":A"#;
        let expected = Ok((
            Call::new(
                Location::test(1, 1),
                CallVariant::library(
                    Identifier::new(Location::test(1, 18), "A".to_owned()),
                    Some(String::from("a.sol")),
                ),
            ),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok() {
        let input = r#"f(bytes2), 10 ether: -> FAILURE
gas legacy: 100("#;
        let expected = Ok((
            Call::new(
                Location::test(1, 1),
                CallVariant::call(
                    Some(Identifier::new(Location::test(1, 1), "f".to_owned())),
                    Some(vec![Type::new(
                        Location::test(1, 3),
                        TypeVariant::bytes(Some(2)),
                    )]),
                    Some(Value::new(
                        Location::test(1, 12),
                        Unit::Ether,
                        "10".to_owned(),
                    )),
                    Some(Vec::new()),
                    Some(Vec::new()),
                    true,
                    Vec::new(),
                ),
            ),
            Some(Token::new(
                Lexeme::Keyword(Keyword::Gas),
                Location::test(2, 1),
            )),
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok_constructor() {
        let input = r#"constructor ->
~ emit E(uint256)"#;
        let expected = Ok((
            Call::new(
                Location::test(1, 1),
                CallVariant::call(
                    Some(Identifier::new(
                        Location::test(1, 1),
                        "constructor".to_owned(),
                    )),
                    None,
                    None,
                    None,
                    Some(Vec::new()),
                    false,
                    vec![Event::new(
                        Location::test(2, 1),
                        EventVariant::signature(
                            Identifier::new(Location::test(2, 8), "E".to_owned()),
                            vec![Type::new(
                                Location::test(2, 10),
                                TypeVariant::integer_unsigned(256),
                            )],
                        ),
                        None,
                        None,
                    )],
                ),
            ),
            Some(Token::new(Lexeme::Eof, Location::test(2, 18))),
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn expected_library_or_call() {
        let input = r#"~"#;
        let expected = Err(ParsingError::Syntax(SyntaxError::new(
            Location::test(1, 1),
            vec!["library", "{identifier}", "("],
            Lexeme::Symbol(Symbol::Tilde),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
