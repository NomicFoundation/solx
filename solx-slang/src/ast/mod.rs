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

use slang_solidity_v2::ast::SourceUnit;

use solx_mlir::Context;

use self::analysis::query::method_identifiers::MethodIdentifiers;
use self::emit::emit_object::EmitObject;
use self::operator_binding::OperatorBindings;

/// Walks a Slang AST and lowers its contract definitions to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits MLIR for the first contract definition in the source unit.
    ///
    /// The current pipeline creates one MLIR module per source file, so
    /// only the first contract is processed. Multi-contract files will be
    /// supported in a future pass.
    ///
    /// Source files containing only interfaces, libraries, or abstract
    /// contracts are skipped without error.
    ///
    /// Returns `Some((contract_name, method_identifiers))` if a contract was
    /// emitted, `None` otherwise.
    ///
    /// A construct the frontend does not yet support panics out of emission; that is deliberate, so
    /// an unhandled case surfaces immediately rather than silently miscompiling.
    pub fn emit(
        &mut self,
        unit: &SourceUnit,
        operator_bindings: &OperatorBindings,
    ) -> Option<(String, BTreeMap<String, String>)> {
        let contracts = unit.contracts();
        let contract = contracts.first()?;

        let name = contract.name().name();
        self.state.operator_bindings = operator_bindings.map.clone();
        contract.emit(self.state, &operator_bindings.functions);

        Some((name, contract.method_identifiers()))
    }
}
