//!
//! The Solidity tests metadata lexical parser.
//!

#[cfg(test)]
mod tests;

pub mod error;
pub mod stream;
pub mod token;

pub use self::error::Error;
pub use self::stream::TokenStream;
pub use self::token::Token;
pub use self::token::lexeme::Lexeme;
pub use self::token::lexeme::keyword::Keyword;
pub use self::token::lexeme::literal::Literal;
pub use self::token::lexeme::literal::boolean::Boolean as BooleanLiteral;
pub use self::token::lexeme::literal::hex::Hex as HexLiteral;
pub use self::token::lexeme::literal::integer::Integer as IntegerLiteral;
pub use self::token::lexeme::literal::string::String as StringLiteral;
pub use self::token::lexeme::symbol::Symbol;
pub use self::token::location::Location;
