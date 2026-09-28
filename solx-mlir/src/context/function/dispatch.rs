//!
//! The internal dispatch attribute a `sol.func` carries.
//!

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;
use slang_solidity_v2::ast::NodeId;

use crate::FunctionKind;

/// The internal dispatch attribute a `sol.func` carries, if any.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FunctionDispatch {
    /// The identifier an internal function pointer dispatches to; never zero, which the dialect
    /// reserves for the null function pointer, since slang numbers nodes from one.
    Identifier(NodeId),
    /// The dialect kind of a constructor, fallback or receive function.
    Kind(FunctionKind),
    /// A synthesized state-variable getter or a base constructor, reached by its symbol alone.
    Symbol,
    /// A modifier, defined as `sol.modifier` and reached by the invocations naming it.
    Modifier,
}

impl FunctionDispatch {
    /// Only the most derived constructor of a hierarchy is the object's creation entry point; every
    /// base constructor is reached by the call the chain emits.
    pub fn new(function: &FunctionDefinition, is_most_derived_constructor: bool) -> Self {
        match function.kind() {
            SlangFunctionKind::Constructor if is_most_derived_constructor => {
                Self::Kind(FunctionKind::Constructor)
            }
            SlangFunctionKind::Constructor => Self::Symbol,
            SlangFunctionKind::Fallback => Self::Kind(FunctionKind::Fallback),
            SlangFunctionKind::Receive => Self::Kind(FunctionKind::Receive),
            SlangFunctionKind::Regular => Self::Identifier(function.node_id()),
            SlangFunctionKind::Modifier => Self::Modifier,
        }
    }
}
