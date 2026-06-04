//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;

use std::collections::HashMap;

use melior::ir::BlockRef;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Context;
use solx_mlir::StateMutability;
use solx_utils::DataLocation;

use self::function::FunctionEmitter;
use self::function::expression::call::type_conversion::TypeConversion;
use self::function::storage_slot::StorageSlot;

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
    /// Returns an error if any function body or constructor initializer
    /// contains unsupported constructs.
    pub fn emit(&mut self, contract: &ContractDefinition) -> anyhow::Result<()> {
        let contract_name = contract.name().name();

        self.pre_register_functions(contract);
        let storage_layout = Self::compute_storage_layout(contract);

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
                &slot.name,
                slot.slot,
                slot.byte_offset,
                element_type,
                &contract_body,
            );
        }

        self.state.current_contract_type = Some(contract_type);
        FunctionEmitter::new(self.state, contract, &storage_layout)
            .emit_constructor(&contract_body)?;
        self.state.current_contract_type = None;

        // Slang's `functions()` filters out Constructor and Modifier kinds.
        for function in contract.functions() {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, contract, &storage_layout)
                .emit_sol(&function, &contract_body)?;
            self.state.current_contract_type = None;
        }

        // Solidity auto-generates an external `view` getter for every public
        // state variable.
        for member in contract.members().iter() {
            let ContractMember::StateVariableDefinition(state_variable) = member else {
                continue;
            };
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            self.emit_state_variable_getter(&state_variable, slot, &contract_body)?;
        }

        Ok(())
    }

    /// Emits the auto-generated external getter for a public, value-typed state
    /// variable: `T public name;` becomes `function name() external view returns
    /// (T)` reading `slot` via `sol.addr_of` + `sol.load`.
    ///
    /// Indexed getters (mappings / arrays, which take key/index arguments) and
    /// reference-typed getters (`string` / `bytes` / struct, which return a
    /// memory copy of the storage value) defer to later domains; they are
    /// skipped here so the rest of the contract still compiles.
    fn emit_state_variable_getter(
        &self,
        state_variable: &StateVariableDefinition,
        slot: &StorageSlot,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some(AbiEntry::Function(abi)) = state_variable.compute_abi_entry() else {
            return Ok(());
        };
        if !abi.inputs().is_empty() {
            return Ok(());
        }
        let declared_type = state_variable
            .get_type()
            .expect("the binder types every state variable");
        if declared_type.is_reference_type() {
            return Ok(());
        }
        let Some(signature) = state_variable.compute_canonical_signature() else {
            return Ok(());
        };
        let Some(selector) = state_variable.compute_selector() else {
            return Ok(());
        };
        let builder = &self.state.builder;
        let element_type = TypeConversion::resolve_slang_type(&declared_type, None, builder);
        let entry = builder.emit_sol_func(
            &signature,
            &[],
            std::slice::from_ref(&element_type),
            Some(selector),
            StateMutability::View,
            None,
            contract_body,
        );
        let pointer_type = builder.types.pointer(element_type, DataLocation::Storage);
        let pointer = builder.emit_sol_addr_of(&slot.name, pointer_type, &entry);
        let value = builder.emit_sol_load(pointer, element_type, &entry)?;
        builder.emit_sol_return(&[value], &entry);
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
    /// Returns a mapping from state variable node ID to its storage slot
    /// (slot index and byte offset within the slot). Returns an empty map
    /// if the ABI is unavailable.
    fn compute_storage_layout(contract: &ContractDefinition) -> HashMap<NodeId, StorageSlot> {
        let Some(abi) = contract.compute_abi() else {
            return HashMap::new();
        };
        abi.storage_layout()
            .iter()
            .map(|item| {
                (
                    item.node_id(),
                    StorageSlot::new(
                        item.slot(),
                        item.offset() as u32,
                        item.label(),
                        item.node_id(),
                    ),
                )
            })
            .collect()
    }
}
