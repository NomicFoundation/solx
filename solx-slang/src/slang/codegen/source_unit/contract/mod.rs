//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub(crate) mod function;

use slang_solidity::backend::ir::ast::ContractDefinition;

use crate::slang::codegen::MlirContext;
use crate::slang::codegen::types::TypeMapper;

use self::function::FunctionEmitter;

/// Lowers a Solidity contract to Sol dialect MLIR.
///
/// Emits `sol.contract` wrapping `sol.func` definitions. The
/// `convert-sol-to-std` pass generates the entry-point dispatcher
/// from the function selectors.
pub(crate) struct ContractEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut MlirContext<'context>,
}

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Creates a new contract emitter.
    pub(crate) fn new(state: &'state mut MlirContext<'context>) -> Self {
        Self { state }
    }

    /// Emits a `sol.contract` containing all function definitions.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub(crate) fn emit(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        let contract_name = contract.name().name();

        self.register_state_variables(contract);
        self.pre_register_functions(contract)?;

        // Emit sol.contract and functions.
        let module_body = self.state.body();
        let contract_body = self.state.emit_sol_contract(&contract_name, &module_body);

        for function in contract.functions() {
            let emitter = FunctionEmitter::new(self.state);
            emitter.emit_sol(&function, &contract_body)?;
        }

        Ok(())
    }

    /// Registers state variables with sequential storage slot assignments.
    fn register_state_variables(&mut self, contract: &ContractDefinition) {
        for (slot, variable) in contract.state_variables().iter().enumerate() {
            self.state
                .register_state_variable(variable.name().name(), slot as u64);
        }
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        for function in contract.functions() {
            let name = FunctionEmitter::mlir_base_name(&function);

            let parameter_types: Vec<String> = function
                .parameters()
                .iter()
                .map(|parameter| TypeMapper::canonical_type(&parameter.type_name()))
                .collect::<anyhow::Result<_>>()?;
            let mlir_name = format!("solx.fn.{name}({})", parameter_types.join(","));

            let has_returns = function.returns().is_some_and(|r| !r.is_empty());

            self.state.register_function_signature(
                &name,
                mlir_name,
                parameter_types.len(),
                has_returns,
            );
        }
        Ok(())
    }
}
