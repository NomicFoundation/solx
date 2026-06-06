//!
//! The resolved MLIR signature of a function.
//!

use melior::ir::Type;

use solx_mlir::StateMutability;

/// The resolved MLIR signature of a function: its symbol name, parameter and
/// result types, public selector, mutability, and MLIR kind.
///
/// Built by `FunctionEmitter::resolve_inner_signature` (the parent module reads
/// these `pub` fields directly — no `pub(crate)` per the recut rules).
pub struct InnerSignature<'context> {
    /// The MLIR symbol the `sol.func` is emitted under.
    pub mlir_name: String,
    /// The Sol-typed parameter types, parallel to the function's parameters.
    ///
    /// For a [`BodyKind::ModifierBody`](super::body_kind::BodyKind::ModifierBody)
    /// emission these are extended with the result types (the wrapped body
    /// receives its return values as trailing parameters); [`Self::parameter_count`]
    /// records the original count before that extension.
    pub mlir_parameter_types: Vec<Type<'context>>,
    /// The number of the function's own parameters — the length of
    /// [`Self::mlir_parameter_types`] before any modifier-body trailing-return
    /// extension. The modifier-body return slots are seeded from the block
    /// arguments at this offset.
    pub parameter_count: usize,
    /// The Sol-typed result types, parallel to the function's returns.
    pub result_types: Vec<Type<'context>>,
    /// The 4-byte public selector, when the function is externally dispatched.
    pub selector: Option<u32>,
    /// The Sol dialect state mutability.
    pub state_mutability: StateMutability,
    /// The Sol dialect function kind (constructor / fallback / receive), or
    /// `None` for a regular function.
    pub mlir_kind: Option<solx_mlir::FunctionKind>,
}
