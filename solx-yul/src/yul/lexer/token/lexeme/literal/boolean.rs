//!
//! The boolean literal lexeme.
//!

use crate::yul::lexer::token::lexeme::keyword::Keyword;

///
/// The boolean literal lexeme.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum Boolean {
    /// Created from the `false` keyword.
    False,
    /// Created from the `true` keyword.
    True,
}

impl Boolean {
    ///
    /// Creates a `false` value.
    ///
    pub fn r#false() -> Self {
        Self::False
    }

    ///
    /// Creates a `true` value.
    ///
    pub fn r#true() -> Self {
        Self::True
    }
}

impl TryFrom<Keyword> for Boolean {
    type Error = Keyword;

    fn try_from(keyword: Keyword) -> Result<Self, Self::Error> {
        Ok(match keyword {
            Keyword::False => Self::False,
            Keyword::True => Self::True,
            unknown => return Err(unknown),
        })
    }
}

impl From<bool> for Boolean {
    fn from(value: bool) -> Self {
        if value { Self::True } else { Self::False }
    }
}

impl std::fmt::Display for Boolean {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::False => write!(f, "false"),
            Self::True => write!(f, "true"),
        }
    }
}
