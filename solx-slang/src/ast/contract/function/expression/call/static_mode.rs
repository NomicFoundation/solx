//!
//! The `StaticMode` external-call mutability discriminant enum.
//!

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
