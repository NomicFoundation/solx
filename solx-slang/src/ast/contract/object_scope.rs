//!
//! The compilation-unit inputs an object's (contract / library) emission consults.
//!

use slang_solidity_v2::ast::FunctionDefinition;

/// The compilation-unit function inputs threaded into object emission, alongside
/// the MLIR `Context`. Kept here rather than on `solx_mlir::Context` so the Slang
/// AST stays off the MLIR builder context. Pure data — `pub` fields, no methods.
pub struct ObjectScope<'state> {
    /// The unit's file-level (free) functions; the object emits the ones it
    /// transitively reaches.
    pub free_functions: &'state [FunctionDefinition],
    /// The unit's operator-bound functions (`using {f as op} for T global;`),
    /// emitted as ordinary internal functions so the operator dispatch resolves.
    pub operator_functions: &'state [FunctionDefinition],
}

impl<'state> ObjectScope<'state> {
    /// Bundles the unit function inputs object emission consults.
    pub fn new(
        free_functions: &'state [FunctionDefinition],
        operator_functions: &'state [FunctionDefinition],
    ) -> Self {
        Self {
            free_functions,
            operator_functions,
        }
    }
}
