//!
//! Describes a pragmatic, target-specific part of the parser.
//!

use std::collections::BTreeSet;
use std::fmt::Debug;

use serde::Deserialize;
use serde::Serialize;

use crate::yul::error::Error;
use crate::yul::lexer::Lexer;
use crate::yul::parser::identifier::Identifier;

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
}

///
/// The root dialect without target-dependent features.
///
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq, PartialEq)]
pub struct DefaultDialect {}

impl Dialect for DefaultDialect {
    type FunctionAttribute = u32;

    fn extract_attributes(
        _identifier: &Identifier,
        _lexer: &mut Lexer,
    ) -> Result<BTreeSet<Self::FunctionAttribute>, Error> {
        Ok(BTreeSet::new())
    }
}
