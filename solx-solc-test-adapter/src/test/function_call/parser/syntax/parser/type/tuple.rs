//!
//! The tuple type parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Symbol;
use crate::test::function_call::parser::lexical::Token;
use crate::test::function_call::parser::lexical::TokenStream;
use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
use crate::test::function_call::parser::syntax::error::ParsingError;
use crate::test::function_call::parser::syntax::parser;
use crate::test::function_call::parser::syntax::parser::r#type::Parser as TypeParser;
use crate::test::function_call::parser::syntax::tree::r#type::Type;
use crate::test::function_call::parser::syntax::tree::r#type::builder::Builder as TypeBuilder;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    ParenthesisLeft,
    /// The `(` has been parsed so far.
    TypeOrParenthesisRight,
    /// The `( {type}` has been parsed so far.
    CommaOrParenthesisRight,
    /// The `( {type},` has been parsed so far.
    Type,
}

///
/// The tuple type parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The token returned from a subparser.
    next: Option<Token>,
    /// The builder of the parsed type.
    builder: TypeBuilder,
}

impl Parser {
    ///
    /// Parses a tuple type literal.
    ///
    /// '(uint8, uint256, bool)'
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        initial: Option<Token>,
    ) -> Result<(Type, Option<Token>), ParsingError> {
        self.next = initial;

        loop {
            match self.state {
                State::ParenthesisLeft => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisLeft),
                            location,
                        } => {
                            self.builder.set_location(location);
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
                            return Ok((self.builder.finish()?, None));
                        }
                        token => {
                            let (element_type, next) =
                                TypeParser::default().parse(stream.clone(), Some(token))?;
                            self.next = next;
                            self.builder.push_tuple_element_type(element_type);
                            self.state = State::CommaOrParenthesisRight;
                        }
                    }
                }
                State::CommaOrParenthesisRight => {
                    match parser::take_or_next(self.next.take(), stream.clone())? {
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::Comma),
                            ..
                        } => self.state = State::Type,
                        Token {
                            lexeme: Lexeme::Symbol(Symbol::ParenthesisRight),
                            ..
                        } => return Ok((self.builder.finish()?, None)),
                        Token { lexeme, location } => {
                            return Err(SyntaxError::new(location, vec![",", ")"], lexeme).into());
                        }
                    }
                }
                State::Type => {
                    let token = parser::take_or_next(self.next.take(), stream.clone())?;
                    let (element_type, next) =
                        TypeParser::default().parse(stream.clone(), Some(token))?;
                    self.next = next;
                    self.builder.push_tuple_element_type(element_type);
                    self.state = State::CommaOrParenthesisRight;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test::function_call::parser::lexical::Lexeme;
    use crate::test::function_call::parser::lexical::Location;
    use crate::test::function_call::parser::lexical::Symbol;
    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;

    use crate::test::function_call::parser::syntax::tree::r#type::Type;
    use crate::test::function_call::parser::syntax::tree::r#type::variant::Variant as TypeVariant;

    #[test]
    fn ok_empty() {
        let input = r#"()"#;

        let expected = Ok((
            Type::new(Location::test(1, 1), TypeVariant::tuple(Vec::new())),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn ok() {
        let input = r#"(uint256, (), bytes[4])"#;

        let expected = Ok((
            Type::new(
                Location::test(1, 1),
                TypeVariant::tuple(vec![
                    Type::new(
                        Location::test(1, 2),
                        TypeVariant::integer_unsigned(solx_utils::BIT_LENGTH_FIELD),
                    ),
                    Type::new(Location::test(1, 11), TypeVariant::tuple(Vec::new())),
                    Type::new(
                        Location::test(1, 15),
                        TypeVariant::array(
                            Type::new(Location::test(1, 15), TypeVariant::bytes(None)),
                            Some("4".to_owned()),
                        ),
                    ),
                ]),
            ),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_comma_or_parenthesis_right() {
        let input = r#"(uint256:"#;

        let expected = Err(ParsingError::Syntax(SyntaxError::new(
            Location::test(1, 9),
            vec![",", ")"],
            Lexeme::Symbol(Symbol::Colon),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
