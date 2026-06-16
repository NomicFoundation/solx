//!
//! The `StaticMode` external-call mutability discriminant enum.
//!

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionMutability;

/// Whether an external call is a STATICCALL.
///
/// A slang-side enum because the `solx-mlir` Builder does not expose a
/// `StaticMode`: the call-emission cluster maps `function.mutability()` into this
/// enum and passes it to the Builder's `static_call` parameter at the
/// `ext_icall` site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaticMode {
    /// A normal external CALL (the callee may mutate state).
    Call,
    /// A STATICCALL (`view` / `pure` callee — no state mutation allowed).
    Static,
}

impl StaticMode {
    /// Classifies an external callee's `STATICCALL` eligibility from its declared
    /// mutability: a `view` / `pure` callee lowers to a `STATICCALL`, anything
    /// else a normal `CALL`.
    pub fn from_function(function_definition: &FunctionDefinition) -> Self {
        match function_definition.mutability() {
            FunctionMutability::View | FunctionMutability::Pure => Self::Static,
            _ => Self::Call,
        }
    }
}
