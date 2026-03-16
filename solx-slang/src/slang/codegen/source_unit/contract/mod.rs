//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub(crate) mod function;

use slang_solidity::backend::ir::ast::ContractDefinition;
use slang_solidity::backend::ir::ast::ContractMember;

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

        // emit_functions takes &self on state (not &mut self), so extract
        // a shared ref to state for the function emitters.
        for member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };

            let emitter = FunctionEmitter::new(self.state);
            emitter.emit_sol(&function, &contract_body)?;
        }

        Ok(())
    }

    /// Registers state variables with sequential storage slot assignments.
    fn register_state_variables(&mut self, contract: &ContractDefinition) {
        let mut slot = 0u64;
        for member in contract.members().iter() {
            if let ContractMember::StateVariableDefinition(variable) = member {
                self.state
                    .register_state_variable(variable.name().name(), slot);
                slot += 1;
            }
        }
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        for member in contract.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };
            let name = function
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "unnamed".to_owned());

            let parameter_types: Vec<String> = function
                .parameters()
                .iter()
                .map(|p| TypeMapper::canonical_type(&p.type_name()))
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
