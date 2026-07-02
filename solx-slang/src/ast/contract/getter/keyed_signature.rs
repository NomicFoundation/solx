//!
//! The resolved signature of a keyed (`mapping`/array) getter.
//!

use melior::ir::Type;

use crate::ast::contract::getter::member::Member;

/// The resolved signature of a keyed getter, built once and shared by the signature query and the
/// getter body so the two can never disagree on the re-walk.
pub struct KeyedSignature<'context> {
    /// The key and index types walked from the variable to the leaf.
    pub input_types: Vec<Type<'context>>,
    /// The leaf's flattened result types.
    pub result_types: Vec<Type<'context>>,
    /// The leaf struct's members, when the leaf is a struct.
    pub members: Option<Vec<Member<'context>>>,
    /// Whether the leaf is a reference (Memory) type.
    pub terminal_is_reference: bool,
}
