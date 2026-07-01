//!
//! The deployable-object emission trait: a contract emits itself as a
//! `sol.contract`.
//!

use solx_mlir::Context;

/// Emits a top-level definition as one deployable `sol.contract`, threading `&mut Context`.
/// Implemented by `ContractDefinition`, emitting its state variables, constructor, and functions.
pub trait EmitObject {
    /// Emits this definition as a deployable `sol.contract` with its functions.
    fn emit(&self, context: &mut Context);
}
