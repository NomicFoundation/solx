//!
//! Yul parser.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::result_large_err)]

pub mod util;
pub mod yul;

pub use yul::error::Error as YulError;
pub use yul::lexer::token::lexeme::literal::Literal as YulLiteral;
pub use yul::lexer::token::lexeme::literal::boolean::Boolean as YulBooleanLiteral;
pub use yul::lexer::token::lexeme::literal::integer::Integer as YulIntegerLiteral;
pub use yul::lexer::token::location::Location as YulLocation;
pub use yul::parser::error::Error as YulParserError;
pub use yul::parser::identifier::Identifier as YulIdentifier;
pub use yul::parser::r#type::Type as YulType;
