//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub(crate) mod function;

use slang_solidity::backend::ir::ast::ContractDefinition;

use solx_mlir::Context;

use self::function::FunctionEmitter;

/// Lowers a Solidity contract to Sol dialect MLIR.
///
/// Emits `sol.contract` wrapping `sol.func` definitions. The
/// `convert-sol-to-std` pass generates the entry-point dispatcher
/// from the function selectors.
pub(crate) struct ContractEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Creates a new contract emitter.
    pub(crate) fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits a `sol.contract` containing all function definitions.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub(crate) fn emit(
        &mut self,
        contract: &ContractDefinition,
        file_identifier: &str,
    ) -> anyhow::Result<()> {
        let contract_name = contract.name().name();

        self.pre_register_functions(contract);
        self.register_state_variables(contract, file_identifier)?;

        // Emit sol.contract and functions.
        let module_body = self.state.body();
        let contract_body = self.state.emit_sol_contract(
            &contract_name,
            // TODO: investigate how other contract kinds (e.g. interface, library) should be represented in MLIR
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        for function in contract.functions() {
            let emitter = FunctionEmitter::new(self.state);
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
            let param_count = function.parameters().len();
            let return_count = function.returns().map_or(0, |returns| returns.len());

            self.state
                .register_function_signature(&name, mlir_name, param_count, return_count);
        }
    }

    /// Registers state variables using slang-solidity's storage layout computation.
    ///
    /// Delegates slot and offset calculation to `compute_abi_with_file_id`,
    /// which accounts for type sizes and storage packing rules.
    fn register_state_variables(
        &mut self,
        contract: &ContractDefinition,
        file_identifier: &str,
    ) -> anyhow::Result<()> {
        let abi = contract
            .compute_abi_with_file_id(file_identifier.to_owned())
            .ok_or_else(|| anyhow::anyhow!("failed to compute ABI for storage layout"))?;
        for item in &abi.storage_layout {
            self.state
                .register_state_variable(item.label.clone(), item.slot as u64);
        }
        Ok(())
    }
}
