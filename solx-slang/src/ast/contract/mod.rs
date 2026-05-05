//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;

use self::function::FunctionEmitter;
use self::function::expression::call::type_conversion::TypeConversion;

/// Lowers a Solidity contract to Sol dialect MLIR.
///
/// Emits `sol.contract` wrapping `sol.func` definitions. The
/// `convert-sol-to-yul` pass generates the entry-point dispatcher
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

    /// Returns whether `contract` is payable (declares a `receive()` function or
    /// a `payable` `fallback()` function). Single source of truth for payability
    /// derivation — used both when emitting the `sol.contract` op and when
    /// resolving `SlangType::Contract` to a `Sol_ContractType`.
    // TODO: walk the inheritance tree like solc does (`receiveFunction` /
    // `fallbackFunction` on `ContractDefinition`, `ContractType::isPayable`)
    // and move this helper into Slang.
    pub fn is_contract_payable(contract: &ContractDefinition) -> bool {
        contract.functions().iter().any(|function| {
            matches!(function.kind(), FunctionKind::Receive)
                || (matches!(function.kind(), FunctionKind::Fallback)
                    && matches!(function.mutability(), FunctionMutability::Payable))
        })
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

        let contract_type = self
            .state
            .builder
            .types
            .contract(&contract_name, Self::is_contract_payable(contract));

        // Emit sol.contract and functions.
        let module_body = self.state.module.body();
        let contract_body = self.state.builder.emit_sol_contract(
            &contract_name,
            // TODO: investigate how other contract kinds (e.g. interface, library) should be represented in MLIR
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        // TODO: emit declarations for inherited state variables once derived
        // contracts compile through this path.
        for member in contract.members().iter() {
            let ContractMember::StateVariableDefinition(state_variable) = member else {
                continue;
            };
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let element_type =
                TypeConversion::resolve_state_variable_type(&state_variable, &self.state.builder)?;
            self.state.builder.emit_sol_state_var(
                &format!("slot_{slot}"),
                *slot,
                element_type,
                &contract_body,
            );
        }

        // Emit the constructor first to align with solc's MLIR layout. Lower
        // the explicit constructor body when the source defines one, otherwise
        // emit an empty stub.
        if let Some(constructor) = contract.constructor() {
            self.state.current_contract_type = Some(contract_type);
            let emitter = FunctionEmitter::new(self.state, &storage_layout);
            emitter.emit_sol(&constructor, &contract_body)?;
            self.state.current_contract_type = None;
        } else {
            let entry = self.state.builder.emit_sol_func(
                "constructor()",
                &[],
                &[],
                None,
                solx_mlir::StateMutability::NonPayable,
                Some(solx_mlir::FunctionKind::Constructor),
                &contract_body,
            );
            self.state.builder.emit_sol_return(&[], &entry);
        }

        // Slang's `functions()` filters out Constructor and Modifier kinds.
        for function in contract.functions() {
            self.state.current_contract_type = Some(contract_type);
            let emitter = FunctionEmitter::new(self.state, &storage_layout);
            emitter.emit_sol(&function, &contract_body)?;
            self.state.current_contract_type = None;
        }

        Ok(())
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) {
        for function in contract.functions() {
            if matches!(function.kind(), FunctionKind::Modifier) {
                continue;
            }
            let mlir_name = FunctionEmitter::mlir_function_name(&function);
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(&function, &self.state.builder);

            self.state.register_function_signature(
                function.node_id(),
                mlir_name,
                parameter_types,
                return_types,
            );
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
        abi.storage_layout()
            .iter()
            .map(|item| (item.node_id(), item.slot() as u64))
            .collect()
    }
}
