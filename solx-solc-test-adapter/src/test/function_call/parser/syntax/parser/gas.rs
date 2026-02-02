//!
//! The gas option parser.
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
use crate::test::function_call::parser::syntax::tree::gas::Gas;
use crate::test::function_call::parser::syntax::tree::gas::builder::Builder as GasBuilder;

///
/// The parser state.
///
#[derive(Debug, Default, Clone, Copy)]
pub enum State {
    /// The initial state.
    #[default]
    Start,
    /// The `gas` has been parsed so far.
    Variant,
    /// The `gas {variant}` has been parsed so far.
    CodeOrColon,
    /// The `gas {variant}` has been parsed so far.
    Colon,
    /// The `gas {variant}:` has been parsed so far.
    Value,
}

///
/// The event parser.
///
#[derive(Default)]
pub struct Parser {
    /// The parser state.
    state: State,
    /// The builder of the parsed value.
    builder: GasBuilder,
    /// The token returned from a subparser.
    next: Option<Token>,
}

impl Parser {
    ///
    /// Parses a gas option.
    ///
    /// '
    /// gas legacy: 10
    /// '
    ///
    pub fn parse(
        mut self,
        stream: Rc<RefCell<TokenStream>>,
        initial: Option<Token>,
    ) -> Result<(Gas, Option<Token>), ParsingError> {
        self.next = initial;

        loop {
            match self.state {
                State::Start => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Gas),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.state = State::Variant;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["gas"], lexeme).into());
                    }
                },
                State::Variant => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme:
                            Lexeme::Keyword(
                                keyword @ Keyword::LegacyOptimized
                                | keyword @ Keyword::Legacy
                                | keyword @ Keyword::IrOptimized
                                | keyword @ Keyword::Ir,
                            ),
                        ..
                    } => {
                        self.builder.set_keyword(keyword);
                        self.state = State::CodeOrColon;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec!["legacyOptimized", "legacy", "irOptimized", "ir"],
                            lexeme,
                        )
                        .into());
                    }
                },
                State::CodeOrColon => match parser::take_or_next(self.next.take(), stream.clone())?
                {
                    Token {
                        lexeme: Lexeme::Keyword(Keyword::Code),
                        location,
                    } => {
                        self.builder.set_location(location);
                        self.state = State::Colon;
                    }
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    } => {
                        self.state = State::Value;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec!["code", ":"], lexeme).into());
                    }
                },
                State::Colon => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme: Lexeme::Symbol(Symbol::Colon),
                        ..
                    } => {
                        self.state = State::Value;
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(location, vec![":"], lexeme).into());
                    }
                },
                State::Value => match parser::take_or_next(self.next.take(), stream.clone())? {
                    Token {
                        lexeme:
                            Lexeme::Literal(LexicalLiteral::Integer(LexicalIntegerLiteral::Decimal {
                                inner: decimal_integer,
                                negative: false,
                            })),
                        ..
                    } => {
                        self.builder.set_value(decimal_integer);
                        return Ok((self.builder.finish(), None));
                    }
                    Token { lexeme, location } => {
                        return Err(SyntaxError::new(
                            location,
                            vec!["{not negative decimal integer literal}"],
                            lexeme,
                        )
                        .into());
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

    use crate::test::function_call::parser::lexical::TokenStream;

    use super::Parser;
    use crate::test::function_call::parser::syntax::error::Error as SyntaxError;
    use crate::test::function_call::parser::syntax::error::ParsingError;
    use crate::test::function_call::parser::syntax::tree::gas::Gas;
    use crate::test::function_call::parser::syntax::tree::gas::variant::Variant;

    #[test]
    fn ok() {
        let input = r#"gas ir: 202020"#;

        let expected = Ok((
            Gas::new(Location::test(1, 1), Variant::ir(), "202020".to_owned()),
            None,
        ));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }

    #[test]
    fn error_expected_gas_variant() {
        let input = r#"gas gas: 10"#;

        let expected = Err(ParsingError::from(SyntaxError::new(
            Location::test(1, 5),
            vec!["legacyOptimized", "legacy", "irOptimized", "ir"],
            Lexeme::Keyword(Keyword::Gas),
        )));

        let result = Parser::default().parse(TokenStream::test(input).wrap(), None);

        assert_eq!(result, expected);
    }
}
