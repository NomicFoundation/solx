//!
//! Describes a pragmatic, target-specific part of the parser.
//!

pub mod era;

use std::collections::BTreeSet;
use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

use solx_yul::yul::error::Error;
use solx_yul::yul::lexer::Lexer;
use solx_yul::yul::lexer::token::location::Location;

use solx_yul::yul::parser::identifier::Identifier;

///
/// Describes a pragmatic, target-specific part of the parser.
///
pub trait Dialect: for<'de> Deserialize<'de> + Serialize + Eq + PartialEq + Debug {
    /// Type of function attributes parsed from their identifiers.
    type FunctionAttribute: for<'de> Deserialize<'de> + Debug + Eq + PartialEq + Serialize + Ord;

    ///
    /// Extractor for the function attributes.
    ///
    fn extract_attributes(
        identifier: &Identifier,
        lexer: &mut Lexer,
    ) -> Result<BTreeSet<Self::FunctionAttribute>, Error>;

    ///
    /// Check the dialect-specific function invariants and potentially modify
    /// their arguments list.
    ///
    fn sanitize_function(
        identifier: &Identifier,
        arguments: &mut Vec<Identifier>,
        location: Location,
        lexer: &mut Lexer,
    ) -> Result<(), Error>;
}
