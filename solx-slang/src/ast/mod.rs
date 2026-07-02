//!
//! Slang AST lowering to MLIR.
//!

/// Pure-Slang semantic queries solx derives from the AST for emission.
pub mod analysis;
/// A produced value paired with the block emission continues in.
pub mod block_and;
/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;
/// Slang AST emission traits.
pub mod emit;
/// User-defined operator bindings (`using {f as op} for T global;`).
pub mod operator_binding;
/// A storable location: an address pointer plus its element type.
pub mod place;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::LibraryDefinition;

use solx_mlir::Context;

use self::analysis::query::method_identifiers::MethodIdentifiers;
use self::emit::emit_object::EmitLibrary;
use self::emit::emit_object::EmitObject;
use self::operator_binding::OperatorBindings;

/// Walks a Slang AST and lowers a single object definition to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits one contract definition as a `sol.contract` object, returning its method identifiers.
    ///
    /// A construct the frontend does not yet support panics out of emission; that is deliberate, so
    /// an unhandled case surfaces immediately rather than silently miscompiling.
    pub fn emit_contract(
        &mut self,
        contract: &ContractDefinition,
        operator_bindings: &OperatorBindings,
        free_functions: &[FunctionDefinition],
    ) -> BTreeMap<String, String> {
        self.state.operator_bindings = operator_bindings.map.clone();
        contract.emit(self.state, &operator_bindings.functions, free_functions);
        contract.method_identifiers()
    }

    /// Emits one library definition as a `sol.contract` object of library kind.
    pub fn emit_library(&mut self, library: &LibraryDefinition) {
        library.emit(self.state);
    }
}
