//!
//! The type variant.
//!

use crate::test::function_call::parser::syntax::Identifier;
use crate::test::function_call::parser::syntax::tree::event::Event;
use crate::test::function_call::parser::syntax::tree::literal::Literal;
use crate::test::function_call::parser::syntax::tree::r#type::Type;
use crate::test::function_call::parser::syntax::tree::value::Value;

///
/// The type variant.
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Variant {
    /// `library` in the source code.
    Library {
        /// The library name.
        identifier: Identifier,
        /// The source file name.
        source: Option<String>,
    },
    /// The function call.
    Call {
        /// The function name.
        identifier: Option<Identifier>,
        /// The params types.
        types: Option<Vec<Type>>,
        /// The value option.
        value: Option<Value>,
        /// The input values.
        input: Option<Vec<Literal>>,
        /// The expected values.
        expected: Option<Vec<Literal>>,
        /// The failure expected flag.
        failure: bool,
        /// The expected events.
        events: Vec<Event>,
    },
}

impl Variant {
    ///
    /// A shortcut constructor.
    ///
    pub fn library(identifier: Identifier, source: Option<String>) -> Self {
        Self::Library { identifier, source }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn call(
        identifier: Option<Identifier>,
        types: Option<Vec<Type>>,
        value: Option<Value>,
        input: Option<Vec<Literal>>,
        expected: Option<Vec<Literal>>,
        failure: bool,
        events: Vec<Event>,
    ) -> Self {
        Self::Call {
            identifier,
            types,
            value,
            input,
            expected,
            failure,
            events,
        }
    }
}
