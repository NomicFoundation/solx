//!
//! The `StaticMode` external-call mutability discriminant enum.
//!

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionMutability;

/// Whether an external call is a STATICCALL.
///
/// The SOLE top-level type of this module (D1). The Rule-12 enum replacing a
/// `static_call: bool` thread through `emit_external_call` (R8-4). A
/// slang-side enum: the frozen `solx-mlir` Builder does not (yet) expose a
/// `StaticMode`, so the call-emission cluster maps `function.mutability()` into
/// this enum and lowers it to the Builder's `static_call` parameter at the
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
