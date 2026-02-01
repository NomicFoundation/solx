//!
//! The value parser.
//!

use std::cell::RefCell;
use std::rc::Rc;

use crate::test::function_call::parser::lexical::IntegerLiteral as LexicalIntegerLiteral;
use crate::test::function_call::parser::lexical::Keyword;
use crate::test::function_call::parser::lexical::Lexeme;
use crate::test::function_call::parser::lexical::Literal as LexicalLiteral;
use crate::test::function_call::parser::lexical::Token;
use crate::test::function_call::parser::lexical::TokenStream;
use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
use crate::test::function_call::parser::syntax::error::ParsingError;
use crate::test::function_call::parser::syntax::parser;
use crate::test::function_call::parser::syntax::tree::value::Value;
use crate::test::function_call::parser::syntax::tree::value::builder::Builder as ValueBuilder;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    Start,
    /// The `{integer literal}` has been parsed so far.
    Unit,
}

///
/// The value parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: ValueBuilder,
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a value.
    ///
    /// '
    /// 10 ether
    /// '
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        initial: Option<Token>,
    ) -> Result<(Value, Option<Token>), ParsingError> {
        self.next = initial;

        loop {
            match self.state {
                State::Start => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme:
                            Lexeme::Literal(LexicalLiteral::Integer(LexicalIntegerLiteral::Decimal {
                                inner: decimal_integer,
                                negative: false,
                            })),
                        location,
                    } => {
                        self.builder.set_amount(decimal_integer);
                        self.builder.set_location(location);
                        self.state = State::Unit;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec!["{positive decimal integer literal}"],
                            lexeme,
                        )
                        .into());
                    }
                },
                State::Unit => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Keyword(keyword @ Keyword::Ether | keyword @ Keyword::Wei),
                        ..
                    } => {
                        self.builder.set_keyword(keyword);
                        return Ok((self.builder.finish(), None));
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["ether", "wei"], lexeme).into());
                    }
                },
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

    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;
    use crate::test::function_call::parser::syntax::tree::value::Value;
    use crate::test::function_call::parser::syntax::tree::value::unit::Unit;

    #[test]
    fn ok() {
        let input = r#"10 ether"#;

        let expected = Ok((
            Value::new(Location::test(1, 1), Unit::Ether, "10".to_string()),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn error_decimal_integer_literal() {
        let input = r#"-5 wei"#;

        let expected = Err(ParsingError::from(SyntaxError::new(
            Location::test(1, 1),
            vec!["{positive decimal integer literal}"],
            Lexeme::Literal(LexicalLiteral::Integer(LexicalIntegerLiteral::new_decimal(
                "5".to_owned(),
                true,
            ))),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
