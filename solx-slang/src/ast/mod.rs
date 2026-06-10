//!
//! Slang AST lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;
pub mod expression_ext;
pub mod operator_binding;
/// Solidity type conversion classification and dispatch.
pub mod type_conversion;

pub use self::expression_ext::ExpressionExt;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Context;

use self::contract::ContractEmitter;
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

    /// Emits MLIR for a single contract definition into this emitter's module.
    ///
    /// Each contract becomes its own deployable object (the caller gives every
    /// contract a fresh MLIR context), with the contract's C3-linearised bases'
    /// state variables and functions pulled in by the contract emitter — so a
    /// multi-contract file emits one complete object per contract, not just the
    /// first.
    ///
    /// `free_functions` is the compilation unit's full set of file-level (free)
    /// functions; the contract emitter pre-registers and emits the ones this
    /// contract reaches. `operator_bindings` is the unit's set of user-defined
    /// operator bindings (`using {f as op} for T global;`), shared across every
    /// contract.
    ///
    /// # Errors
    ///
    /// Returns an error if code generation encounters unsupported constructs.
    /// Returns the contract's name and its public-method selector table.
    pub fn emit_contract(
        &mut self,
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
        operator_bindings: &OperatorBindings,
    ) -> anyhow::Result<(String, BTreeMap<String, String>)> {
        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract, free_functions, operator_bindings)?;

        let mut method_identifiers = BTreeMap::new();
        for contract_member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = contract_member else {
                continue;
            };
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }

        Ok((name, method_identifiers))
    }
}
