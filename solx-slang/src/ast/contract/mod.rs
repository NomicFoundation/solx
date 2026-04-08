//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;

use std::collections::HashMap;

use slang_solidity::backend::ir::ast::ContractDefinition;
use slang_solidity::cst::NodeId;

use solx_mlir::Context;

use self::function::FunctionEmitter;

/// Lowers a Solidity contract to Sol dialect MLIR.
///
/// Emits `sol.contract` wrapping `sol.func` definitions. The
/// `convert-sol-to-std` pass generates the entry-point dispatcher
/// from the function selectors.
pub struct ContractEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Creates a new contract emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits a `sol.contract` containing all function definitions.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub fn emit(
        &mut self,
        contract: &ContractDefinition,
        file_identifier: &str,
    ) -> anyhow::Result<()> {
        let contract_name = contract.name().name();

        self.pre_register_functions(contract);
        let storage_layout = Self::compute_storage_layout(contract, file_identifier);

        // Emit sol.contract and functions.
        let module_body = self.state.module.body();
        let contract_body = self.state.builder.emit_sol_contract(
            &contract_name,
            // TODO: investigate how other contract kinds (e.g. interface, library) should be represented in MLIR
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        // Emit sol.state_var declarations for each storage slot.
        for slot in storage_layout.values() {
            self.state
                .builder
                .emit_sol_state_var(&format!("slot_{slot}"), *slot, &contract_body);
        }

        for function in contract.functions() {
            let emitter = FunctionEmitter::new(self.state, &storage_layout);
            emitter.emit_sol(&function, &contract_body)?;
        }

        Ok(())
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) {
        for function in contract.functions() {
            let name = FunctionEmitter::mlir_base_name(&function);
            let mlir_name = FunctionEmitter::mlir_function_name(&function);
            let parameter_count = function.parameters().len();
            let return_count = function.returns().map_or(0, |returns| returns.len());

            self.state
                .register_function_signature(&name, mlir_name, parameter_count, return_count);
        }
    }

    /// Computes the storage layout using slang-solidity's ABI computation.
    ///
    /// Returns a mapping from state variable node ID to storage slot.
    /// Returns an empty map if the ABI is unavailable.
    fn compute_storage_layout(
        contract: &ContractDefinition,
        file_identifier: &str,
    ) -> HashMap<NodeId, u64> {
        let Some(abi) = contract.compute_abi_with_file_id(file_identifier.to_owned()) else {
            return HashMap::new();
        };
        abi.storage_layout
            .iter()
            .map(|item| (item.node_id, item.slot as u64))
            .collect()
    }
}
