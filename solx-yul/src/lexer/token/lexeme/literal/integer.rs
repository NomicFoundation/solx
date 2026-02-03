//!
//! The integer literal lexeme.
//!

use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::Literal;
use crate::lexer::token::location::Location;

///
/// The integer literal lexeme.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum Integer {
    /// An integer literal, like `42`.
    Decimal {
        /// The inner literal contents.
        inner: String,
    },
    /// A hexadecimal literal, like `0xffff`.
    Hexadecimal {
        /// The inner literal contents.
        inner: String,
    },
}

impl Integer {
    /// Base for decimal numbers.
    const BASE_DECIMAL: u32 = 10;

    /// Base for hexadecimal numbers.
    const BASE_HEXADECIMAL: u32 = 16;

    ///
    /// Creates a decimal value.
    ///
    pub fn new_decimal(inner: String) -> Self {
        Self::Decimal { inner }
    }

    ///
    /// Creates a hexadecimal value.
    ///
    pub fn new_hexadecimal(inner: String) -> Self {
        Self::Hexadecimal { inner }
    }

    ///
    /// Parses the value from the source code slice.
    ///
    pub fn parse(input: &str) -> Option<Token> {
        let (value, length) = if let Some(body) = input.strip_prefix("0x") {
            let end = body
                .find(Self::cannot_continue_hexadecimal)
                .unwrap_or(body.len());
            let length = "0x".len() + end;
            let value = Self::new_hexadecimal(input[..length].to_owned());
            (value, length)
        } else if input.starts_with(Self::can_begin_decimal) {
            let end = input
                .find(Self::cannot_continue_decimal)
                .unwrap_or(input.len());
            let length = end;
            let value = Self::new_decimal(input[..length].to_owned());
            (value, length)
        } else {
            return None;
        };

        let token = Token::new(
            Location::new(0, length),
            Lexeme::Literal(Literal::Integer(value)),
            length,
        );
        Some(token)
    }

    ///
    /// Checks whether the character can begin a decimal number.
    ///
    pub fn can_begin_decimal(character: char) -> bool {
        Self::can_continue_decimal(character)
    }

    ///
    /// Checks whether the character can continue a decimal number.
    ///
    pub fn can_continue_decimal(character: char) -> bool {
        character.is_digit(Self::BASE_DECIMAL)
    }

    ///
    /// Checks whether the character cannot continue a decimal number.
    ///
    pub fn cannot_continue_decimal(character: char) -> bool {
        !Self::can_continue_decimal(character)
    }

    ///
    /// Checks whether the character can continue a hexadecimal number.
    ///
    pub fn can_continue_hexadecimal(character: char) -> bool {
        character.is_digit(Self::BASE_HEXADECIMAL)
    }

    ///
    /// Checks whether the character cannot continue a hexadecimal number.
    ///
    pub fn cannot_continue_hexadecimal(character: char) -> bool {
        !Self::can_continue_hexadecimal(character)
    }
}

impl std::fmt::Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decimal { inner } => write!(f, "{inner}"),
            Self::Hexadecimal { inner } => write!(f, "{inner}"),
        }
    }
}
