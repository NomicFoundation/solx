//!
//! The deployable-object emission trait: a contract or library emits itself as a
//! `sol.contract`.
//!

use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Context;

/// Emits a top-level definition as one deployable `sol.contract`, threading `&mut Context`.
/// Implemented by `ContractDefinition`, emitting its state variables, constructor, and functions.
pub trait EmitObject {
    /// Emits this definition as a deployable `sol.contract` with its functions, plus the reachable
    /// free functions it references. `operator_functions` are the operator-bound free functions
    /// (`using {f as op} for T global;`) that seed reachability; `free_functions` is the source-unit
    /// pool the walk selects from. Both are emitted as ordinary internal functions under their
    /// node-id-qualified symbols so the operator dispatch and pointer references resolve.
    fn emit(
        &self,
        context: &mut Context,
        operator_functions: &[FunctionDefinition],
        free_functions: &[FunctionDefinition],
    );
}

/// Emits a library as one deployable `sol.contract` of library kind, threading `&mut Context`.
/// Implemented by `LibraryDefinition`, emitting its `external` / `public` functions.
pub trait EmitLibrary {
    /// Emits this library as a deployable `sol.contract` of library kind carrying its
    /// externally-visible functions, so the `// library:` directive can deploy and link it.
    fn emit(&self, context: &mut Context);
}
