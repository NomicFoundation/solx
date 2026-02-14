//!
//! The event signature variant.
//!

use crate::test::function_call::parser::syntax::Identifier;
use crate::test::function_call::parser::syntax::Type;

///
/// The event signature variant.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variant {
    /// `<anonymous>` in the source code.
    Anonymous,
    /// `{identifier}({types})` in the source code.
    Signature {
        /// The function name.
        identifier: Identifier,
        /// The function input types.
        types: Vec<Type>,
    },
}

impl Variant {
    ///
    /// A shortcut constructor.
    ///
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn signature(identifier: Identifier, types: Vec<Type>) -> Self {
        Self::Signature { identifier, types }
    }
}
