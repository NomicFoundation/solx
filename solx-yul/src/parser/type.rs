//!
//! The Yul source code type.
//!

use solx_codegen_evm::IContext;

use crate::error::Error;
use crate::lexer::Lexer;
use crate::lexer::token::Token;
use crate::lexer::token::lexeme::Lexeme;
use crate::lexer::token::lexeme::keyword::Keyword;
use crate::parser::error::Error as ParserError;

///
/// The Yul source code type.
///
/// The type is not currently in use, so all values have the `uint256` type by default.
///
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq)]
pub enum Type {
    /// The `bool` type.
    Bool,
    /// The `int{N}` type.
    Int(usize),
    /// The `uint{N}` type.
    UInt(usize),
    /// The custom user-defined type.
    Custom(String),
}

impl Default for Type {
    fn default() -> Self {
        Self::UInt(Self::DEFAULT_BIT_LENGTH)
    }
}

impl Type {
    /// Bit length of a default integer type.
    /// Mirrors the `BIT_LENGTH_FIELD` from `era_compiler_common` crate.
    /// In future, if more dialects are supported, this should be moved to the
    /// `Dialect` trait.
    const DEFAULT_BIT_LENGTH: usize = 256;

    ///
    /// The element parser.
    ///
    pub fn parse(lexer: &mut Lexer, initial: Option<Token>) -> Result<Self, Error> {
        let token = crate::parser::take_or_next(initial, lexer)?;

        match token {
            Token {
                lexeme: Lexeme::Keyword(Keyword::Bool),
                ..
            } => Ok(Self::Bool),
            Token {
                lexeme: Lexeme::Keyword(Keyword::Int(bitlength)),
                ..
            } => Ok(Self::Int(bitlength)),
            Token {
                lexeme: Lexeme::Keyword(Keyword::Uint(bitlength)),
                ..
            } => Ok(Self::UInt(bitlength)),
            Token {
                lexeme: Lexeme::Identifier(identifier),
                ..
            } => Ok(Self::Custom(identifier.inner)),
            token => Err(ParserError::InvalidToken {
                location: token.location,
                expected: vec!["{type}"],
                found: token.lexeme.to_string(),
            }
            .into()),
        }
    }

    ///
    /// Converts the type into its LLVM representation.
    ///
    pub fn into_llvm<'ctx, C>(self, context: &C) -> inkwell::types::IntType<'ctx>
    where
        C: IContext<'ctx>,
    {
        match self {
            Self::Bool => context.integer_type(solx_utils::BIT_LENGTH_BOOLEAN),
            Self::Int(bitlength) => context.integer_type(bitlength),
            Self::UInt(bitlength) => context.integer_type(bitlength),
            Self::Custom(_) => context.field_type(),
        }
    }
}
