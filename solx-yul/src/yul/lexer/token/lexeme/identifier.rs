//!
//! The identifier lexeme.
//!

use crate::yul::lexer::token::Token;
use crate::yul::lexer::token::lexeme::Lexeme;
use crate::yul::lexer::token::lexeme::keyword::Keyword;
use crate::yul::lexer::token::location::Location;

///
/// The identifier lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    /// The inner string.
    pub inner: String,
}

impl Identifier {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(inner: String) -> Self {
        Self { inner }
    }

    ///
    /// Parses the identifier, returning it as a token.
    ///
    pub fn parse(input: &str) -> Option<Token> {
        if !input.starts_with(Self::can_begin) {
            return None;
        }
        let end = input.find(Self::cannot_continue).unwrap_or(input.len());

        let inner = input[..end].to_string();
        let length = inner.len();

        if let Some(token) = Keyword::parse(inner.as_str()) {
            return Some(token);
        }

        Some(Token::new(
            Location::new(0, length),
            Lexeme::Identifier(Self::new(inner)),
            length,
        ))
    }

    ///
    /// Checks whether the character can begin an identifier.
    ///
    pub fn can_begin(character: char) -> bool {
        character.is_alphabetic() || character == '_' || character == '$'
    }

    ///
    /// Checks whether the character can continue an identifier.
    ///
    pub fn can_continue(character: char) -> bool {
        Self::can_begin(character)
            || character.is_numeric()
            || character == '_'
            || character == '$'
            || character == '.'
    }

    ///
    /// Checks whether the character cannot continue an identifier.
    ///
    pub fn cannot_continue(character: char) -> bool {
        !Self::can_continue(character)
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
