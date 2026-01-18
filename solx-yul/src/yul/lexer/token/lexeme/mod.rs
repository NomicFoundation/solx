//!
//! The lexeme.
//!

pub mod comment;
pub mod identifier;
pub mod keyword;
pub mod literal;
pub mod symbol;

use self::identifier::Identifier;
use self::keyword::Keyword;
use self::literal::Literal;
use self::symbol::Symbol;

///
/// The lexeme.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Lexeme {
    /// The keyword lexeme.
    Keyword(Keyword),
    /// The symbol lexeme.
    Symbol(Symbol),
    /// The identifier lexeme.
    Identifier(Identifier),
    /// The literal lexeme.
    Literal(Literal),
    /// The single-line comment lexeme.
    SingleLineComment(String),
    /// The multi-line comment lexeme. (unused)
    MultiLineComment,
    /// The end-of-file lexeme.
    EndOfFile,
}

impl std::fmt::Display for Lexeme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Keyword(inner) => write!(f, "{inner}"),
            Self::Symbol(inner) => write!(f, "{inner}"),
            Self::Identifier(inner) => write!(f, "{inner}"),
            Self::Literal(inner) => write!(f, "{inner}"),
            Self::SingleLineComment(inner) => write!(f, "{inner}"),
            Self::MultiLineComment => Ok(()),
            Self::EndOfFile => write!(f, "EOF"),
        }
    }
}
