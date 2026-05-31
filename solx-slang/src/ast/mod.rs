//!
//! Slang AST lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;

use solx_mlir::Context;

use self::contract::ContractEmitter;

/// Walks a Slang AST and lowers a single contract definition to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits MLIR for `contract` and returns its name and method-identifier
    /// table.
    ///
    /// One [`Context`] holds one contract's MLIR module, so the caller is
    /// expected to iterate the source unit's contracts and call this with a
    /// fresh [`Context`] per contract.
    ///
    /// # Errors
    ///
    /// Returns an error if code generation encounters unsupported constructs.
    pub fn emit(
        &mut self,
        contract: &ContractDefinition,
    ) -> anyhow::Result<(String, BTreeMap<String, String>)> {
        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract)?;

        let mut method_identifiers = BTreeMap::new();
        // Walk the inheritance-linearised function list so derived
        // contracts expose inherited externals in their ABI.
        for function in contract.compute_linearised_functions() {
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        for contract_member in contract.members().iter() {
            if let ContractMember::StateVariableDefinition(state_variable) = contract_member {
                let Some(signature) = state_variable.compute_canonical_signature() else {
                    continue;
                };
                let Some(selector) = state_variable.compute_selector() else {
                    continue;
                };
                method_identifiers.insert(signature, format!("{selector:08x}"));
            }
        }

        Ok((name, method_identifiers))
    }
}
