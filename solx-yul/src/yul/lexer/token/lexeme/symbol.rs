//!
//! The symbol lexeme.
//!

use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::location::Location;

///
/// The symbol lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Symbol {
    /// The `:=` symbol.
    Assignment,
    /// The `->` symbol.
    Arrow,
    /// The `{` symbol.
    BracketCurlyLeft,
    /// The `}` symbol.
    BracketCurlyRight,
    /// The `(` symbol.
    ParenthesisLeft,
    /// The `)` symbol.
    ParenthesisRight,
    /// The `,` symbol.
    Comma,
    /// The `:` symbol.
    Colon,
}

impl Symbol {
    ///
    /// Parses the symbol, returning it as a token.
    ///
    pub fn parse(input: &str) -> Option<Token> {
        let (symbol, length) = match &input[..std::cmp::min(input.len(), 2)] {
            ":=" => (Self::Assignment, 2),
            "->" => (Self::Arrow, 2),

            _ => match &input[..std::cmp::min(input.len(), 1)] {
                "{" => (Self::BracketCurlyLeft, 1),
                "}" => (Self::BracketCurlyRight, 1),
                "(" => (Self::ParenthesisLeft, 1),
                ")" => (Self::ParenthesisRight, 1),
                "," => (Self::Comma, 1),
                ":" => (Self::Colon, 1),

                _ => return None,
            },
        };

        Some(Token::new(
            Location::new(0, length),
            Lexeme::Symbol(symbol),
            length,
        ))
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assignment => write!(f, ":="),
            Self::Arrow => write!(f, "->"),
            Self::BracketCurlyLeft => write!(f, "{{"),
            Self::BracketCurlyRight => write!(f, "}}"),
            Self::ParenthesisLeft => write!(f, "("),
            Self::ParenthesisRight => write!(f, ")"),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
        }
    }
}
