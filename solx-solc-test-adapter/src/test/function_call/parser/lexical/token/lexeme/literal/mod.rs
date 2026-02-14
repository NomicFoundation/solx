//!
//! The lexical token literal lexeme.
//!

pub mod boolean;
pub mod hex;
pub mod integer;
pub mod string;

use std::fmt;

use self::hex::Hex;
use self::integer::Integer;
use self::string::String;

///
/// The lexical literal.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Literal {
    /// A boolean literal, like `true`, or `false`.
    Boolean(bool),
    /// An integer literal, like `42`, or `0xff`.
    Integer(Integer),
    /// A string literal, like `"message"`.
    String(String),
    /// A hex literal, like `hex"1234"`.
    Hex(Hex),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(inner) => write!(f, "{inner}"),
            Self::Integer(inner) => write!(f, "{inner}"),
            Self::String(inner) => write!(f, "{inner}"),
            Self::Hex(inner) => write!(f, "{inner}"),
        }
    }
}
