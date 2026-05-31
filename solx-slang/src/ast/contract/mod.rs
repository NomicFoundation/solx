//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;

use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;
use solx_mlir::StateMutability;

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

        // Walk the C3-linearised state-variable list so derived contracts
        // pick up base-contract storage slots and getters in addition to
        // their own.
        let mut emitted_slots: std::collections::HashSet<(U256, u32)> =
            std::collections::HashSet::new();
        for state_variable in contract.compute_linearised_state_variables() {
            let Some(&(slot, byte_offset)) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            if !emitted_slots.insert((slot, byte_offset)) {
                // Distinct state variables may share a (slot, offset) only
                // through inheritance re-linearisation; emit each symbol once.
                continue;
            }
            let element_type =
                TypeConversion::resolve_state_variable_type(&state_variable, &self.state.builder)?;
            self.state.builder.emit_sol_state_var(
                &Self::storage_symbol(slot, byte_offset),
                slot,
                byte_offset,
                element_type,
                &contract_body,
            );
        }

        self.state.current_contract_type = Some(contract_type);
        FunctionEmitter::new(self.state, contract, &storage_layout)
            .emit_constructor(&contract_body)?;
        self.state.current_contract_type = None;

        // `compute_linearised_functions` walks the C3-linearised inheritance
        // chain so derived contracts pick up base-contract methods (subject
        // to override resolution).
        for function in contract.compute_linearised_functions() {
            if matches!(
                function.kind(),
                FunctionKind::Constructor | FunctionKind::Modifier
            ) {
                continue;
            }
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, contract, &storage_layout)
                .emit_sol(&function, &contract_body)?;
            self.state.current_contract_type = None;
        }

        for state_variable in contract.compute_linearised_state_variables() {
            if matches!(state_variable.mutability(), StateVariableMutability::Constant) {
                self.emit_constant_getter(&state_variable, &contract_body)?;
            } else if let Some(&(slot, byte_offset)) =
                storage_layout.get(&state_variable.node_id())
            {
                self.emit_state_variable_getter(
                    &state_variable,
                    slot,
                    byte_offset,
                    &contract_body,
                )?;
            }
        }

        Ok(())
    }

    /// Builds the storage-variable symbol name for a `(slot, byte_offset)`
    /// location. Packed small-value variables share a slot but differ in
    /// byte offset, so the offset is part of the symbol.
    pub(crate) fn storage_symbol(slot: U256, byte_offset: u32) -> String {
        format!("slot_{slot}_{byte_offset}")
    }

    /// Emits the auto-generated external getter for a `public constant` state
    /// variable. Only direct integer / address literals are supported; more
    /// elaborate constant expressions need the full expression emitter.
    fn emit_constant_getter(
        &self,
        state_variable: &StateVariableDefinition,
        contract_body: &melior::ir::BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some(AbiEntry::Function(abi)) = state_variable.compute_abi_entry() else {
            return Ok(());
        };
        if !abi.inputs().is_empty() {
            return Ok(());
        }
        let Some(signature) = state_variable.compute_canonical_signature() else {
            return Ok(());
        };
        let Some(selector) = state_variable.compute_selector() else {
            return Ok(());
        };
        let Some(initializer) = state_variable.value() else {
            return Ok(());
        };

        let value = match &initializer {
            Expression::DecimalNumberExpression(decimal) => decimal.integer_value(),
            Expression::HexNumberExpression(hex) => hex.integer_value(),
            _ => initializer.get_type().and_then(|slang_type| match slang_type {
                SlangType::Literal(literal) => match literal.kind() {
                    LiteralKind::Integer { value } => Some(value),
                    LiteralKind::HexInteger { value, .. } => Some(value),
                    _ => None,
                },
                _ => None,
            }),
        };
        let Some(value) = value else {
            return Ok(());
        };

        let builder = &self.state.builder;
        let element_type = TypeConversion::resolve_state_variable_type(state_variable, builder)?;
        let entry = builder.emit_sol_func(
            &signature,
            &[],
            std::slice::from_ref(&element_type),
            Some(selector),
            StateMutability::Pure,
            None,
            contract_body,
        );
        let constant = builder.emit_constant(&value, element_type, &entry);
        builder.emit_sol_return(&[constant], &entry);
        Ok(())
    }

    /// Emits the auto-generated external getter for a public state variable.
    ///
    /// Scalar `T public name;` becomes `function name() external view
    /// returns (T)` reading slot `slot`. Array/mapping/struct getters
    /// require indexed access and are not yet emitted; they are silently
    /// skipped here so the rest of the contract still compiles.
    fn emit_state_variable_getter(
        &self,
        state_variable: &StateVariableDefinition,
        slot: U256,
        byte_offset: u32,
        contract_body: &melior::ir::BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some(AbiEntry::Function(abi)) = state_variable.compute_abi_entry() else {
            return Ok(());
        };
        // Scalar getters only for now — indexed forms need sol.gep / sol.map.
        if !abi.inputs().is_empty() {
            return Ok(());
        }
        let Some(signature) = state_variable.compute_canonical_signature() else {
            return Ok(());
        };
        let Some(selector) = state_variable.compute_selector() else {
            return Ok(());
        };

        let declared_type = state_variable.get_type().ok_or_else(|| {
            anyhow::anyhow!("unresolved type for state variable getter")
        })?;
        let builder = &self.state.builder;
        let element_type = TypeConversion::resolve_state_variable_type(state_variable, builder)?;
        // A reference-typed state variable (`string`/`bytes`/array/struct) is
        // addressed by the reference type itself in storage; value types use
        // a `!sol.ptr<T, Storage>`. Matching the address type the initializer
        // uses keeps the `sol.addr_of` symbol consistent.
        let address_type = if declared_type.is_reference_type() {
            element_type
        } else {
            builder
                .types
                .pointer(element_type, solx_utils::DataLocation::Storage)
        };
        let entry = builder.emit_sol_func(
            &signature,
            &[],
            std::slice::from_ref(&element_type),
            Some(selector),
            StateMutability::View,
            None,
            contract_body,
        );
        let slot_name = Self::storage_symbol(slot, byte_offset);
        let storage_ref = builder.emit_sol_addr_of(&slot_name, address_type, &entry);
        let value = if declared_type.is_reference_type() {
            // The storage reference is the value the ABI encoder reads from.
            storage_ref
        } else {
            builder.emit_sol_load(storage_ref, element_type, &entry)?
        };
        builder.emit_sol_return(&[value], &entry);
        Ok(())
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) {
        for function in contract.compute_linearised_functions() {
            if matches!(
                function.kind(),
                FunctionKind::Constructor | FunctionKind::Modifier
            ) {
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
    fn compute_storage_layout(contract: &ContractDefinition) -> HashMap<NodeId, (U256, u32)> {
        let mut layout: HashMap<NodeId, (U256, u32)> = HashMap::new();
        if let Some(abi) = contract.compute_abi() {
            for item in abi.storage_layout().iter() {
                let byte_offset = u32::try_from(item.offset()).unwrap_or(0);
                layout.insert(item.node_id(), (item.slot(), byte_offset));
            }
        }
        // Slang's ABI omits `immutable` state variables (they live in code,
        // not storage). For the experimental Slang frontend we treat them as
        // ordinary storage variables so that compilation succeeds — runtime
        // behaviour around code immutability won't match solc's, but the
        // observable semantics (read after constructor write) survives.
        let mut next_slot: U256 = layout
            .values()
            .map(|(slot, _)| *slot)
            .max()
            .map(|max| max + U256::from(1))
            .unwrap_or(U256::from(0));
        for state_variable in contract.compute_linearised_state_variables() {
            if !matches!(
                state_variable.mutability(),
                StateVariableMutability::Immutable
            ) {
                continue;
            }
            if layout.contains_key(&state_variable.node_id()) {
                continue;
            }
            layout.insert(state_variable.node_id(), (next_slot, 0));
            next_slot += U256::from(1);
        }
        layout
    }
}
