//!
//! Source unit (top-level file) lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub(crate) mod contract;

use std::collections::BTreeMap;

use slang_solidity::backend::abi::AbiEntry;
use slang_solidity::backend::ir::ast::ContractMember;
use slang_solidity::backend::ir::ast::SourceUnit;

use solx_mlir::Context;

use self::contract::ContractEmitter;

/// Walks a `SourceUnit` and lowers its contract definitions to MLIR.
pub(crate) struct SourceUnitEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> SourceUnitEmitter<'state, 'context> {
    /// Creates a new source unit emitter.
    pub(crate) fn new(state: &'state mut Context<'context>) -> Self {
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
    /// # Errors
    ///
    /// Returns an error if code generation encounters unsupported constructs.
    /// Returns `Some(contract_name)` if a contract was emitted, `None` otherwise.
    pub(crate) fn emit(
        &mut self,
        unit: &SourceUnit,
    ) -> anyhow::Result<Option<(String, BTreeMap<String, String>)>> {
        let contracts = unit.contracts();
        let Some(contract) = contracts.first() else {
            return Ok(None);
        };

        for skipped in contracts.iter().skip(1) {
            eprintln!(
                "warning: skipping contract '{}': only one contract per source file is supported",
                skipped.name().name()
            );
        }

        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract)?;

        let mut method_identifiers = BTreeMap::new();
        for contract_member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = contract_member else {
                continue;
            };
            let Some(AbiEntry::Function { name, inputs, .. }) = function.compute_abi_entry() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            let param_types: Vec<&str> = inputs.iter().map(|input| input.r#type.as_str()).collect();
            let signature = format!("{name}({})", param_types.join(","));
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }

        Ok(Some((name, method_identifiers)))
    }
}
