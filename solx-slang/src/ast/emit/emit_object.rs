//!
//! The deployable-object emission trait: a contract emits itself as a
//! `sol.contract`.
//!

use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Context;

/// Emits a top-level definition as one deployable `sol.contract`, threading `&mut Context`.
/// Implemented by `ContractDefinition`, emitting its state variables, constructor, and functions.
pub trait EmitObject {
    /// Emits this definition as a deployable `sol.contract` with its functions, plus the
    /// operator-bound free functions it dispatches to (`using {f as op} for T global;`), emitted as
    /// ordinary internal functions so the operator dispatch resolves.
    fn emit(&self, context: &mut Context, operator_functions: &[FunctionDefinition]);
}
