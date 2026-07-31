//!
//! The attribute a `sol.func` is reached through.
//!

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind as SlangFunctionKind;
use slang_solidity_v2::ast::NodeId;

use crate::FunctionKind;

/// The attribute a `sol.func` is reached through. A dispatch identifier is never zero, which the
/// dialect reserves for the null function pointer, since slang numbers nodes from one.
#[derive(Clone, Copy)]
pub enum FunctionDispatch {
    /// The identifier an internal function pointer dispatches to.
    Identifier(NodeId),
    /// The dialect kind of a constructor, fallback or receive function.
    Kind(FunctionKind),
}

impl From<&FunctionDefinition> for FunctionDispatch {
    fn from(function: &FunctionDefinition) -> Self {
        match function.kind() {
            SlangFunctionKind::Constructor => Self::Kind(FunctionKind::Constructor),
            SlangFunctionKind::Fallback => Self::Kind(FunctionKind::Fallback),
            SlangFunctionKind::Receive => Self::Kind(FunctionKind::Receive),
            SlangFunctionKind::Regular => Self::Identifier(function.node_id()),
            SlangFunctionKind::Modifier => unreachable!("slang yields no modifier as a function"),
        }
    }
}
