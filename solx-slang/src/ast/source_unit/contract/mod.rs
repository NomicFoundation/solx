//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub(crate) mod function;

use slang_solidity::backend::ir::ast::ContractDefinition;
use slang_solidity::backend::ir::ast::ElementaryType;
use slang_solidity::backend::ir::ast::TypeName;

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
    pub(crate) fn emit(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        let contract_name = contract.name().name();

        self.pre_register_functions(contract);
        self.register_state_variables(contract)?;

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

    /// Registers state variables with sequential storage slot assignments.
    ///
    /// Each variable gets its own 256-bit storage slot. Sub-32-byte types
    /// (e.g. `uint8`, `bool`, `address`) that would be packed by solc are
    /// rejected because the sequential layout would produce incorrect
    /// storage reads/writes.
    fn register_state_variables(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        // TODO: check if slang-solidity can provide storage layout information
        for (slot, variable) in contract.state_variables().iter().enumerate() {
            let name = variable.name().name();
            let type_name = variable.type_name();
            // TODO: implement storage packing and remove this restriction
            if !Self::is_full_slot_type(&type_name) {
                anyhow::bail!(
                    "state variable '{name}' has sub-32-byte type; \
                     storage packing is not yet implemented"
                );
            }
            self.state.register_state_variable(name, slot as u64);
        }
        Ok(())
    }

    /// Returns whether a Solidity type occupies a full 256-bit storage slot.
    /// TODO: can be moved to slang-solidity
    fn is_full_slot_type(type_name: &TypeName) -> bool {
        match type_name {
            TypeName::ElementaryType(elementary) => {
                matches!(
                    elementary,
                    ElementaryType::UintKeyword(t) if t.text == "uint256"
                ) || matches!(
                    elementary,
                    ElementaryType::IntKeyword(t) if t.text == "int256"
                ) || matches!(
                    elementary,
                    ElementaryType::BytesKeyword(t) if t.text == "bytes32"
                )
            }
            TypeName::MappingType(_) => true,
            _ => false,
        }
    }
}
