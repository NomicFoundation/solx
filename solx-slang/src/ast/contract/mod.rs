//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;
mod library;

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

use melior::ir::BlockLike;
use melior::ir::Type;
use melior::ir::Value;

use solx_mlir::Context;
use solx_mlir::StateMutability;

use self::function::FunctionEmitter;
use self::function::expression::call::type_conversion::TypeConversion;
use self::function::storage_slot::StorageSlot;

/// Maps each state variable's node ID to its storage location: the slot, the
/// byte offset within the slot, and the data location (persistent `Storage`
/// or `Transient`). The data location selects SLOAD/SSTORE versus TLOAD/TSTORE
/// access and keeps transient symbols distinct from storage symbols (the two
/// address spaces have independent, potentially colliding slot numbering).
pub(crate) type StorageLayout = HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>;

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

    /// Returns whether `contract` is payable (a `receive()` function or a
    /// `payable` `fallback()`, declared on the contract or inherited). Single
    /// source of truth for payability derivation — used both when emitting the
    /// `sol.contract` op and when resolving `SlangType::Contract` to a
    /// `Sol_ContractType`. Walks the C3-linearised function set so a `receive`
    /// inherited from a base marks the deriving contract payable, matching solc.
    pub fn is_contract_payable(contract: &ContractDefinition) -> bool {
        contract.compute_linearised_functions().iter().any(|function| {
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

        // Internal library functions called by the contract (`L.f(...)`) are
        // not part of `compute_linearised_functions`, so pre-register them and
        // emit their bodies into this contract's module below — they lower like
        // ordinary internal functions.
        let library_functions = library::collect_library_functions(contract);
        for library_function in &library_functions {
            let mlir_name = FunctionEmitter::mlir_function_name(library_function);
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(library_function, &self.state.builder);
            self.state.register_function_signature(
                library_function.node_id(),
                mlir_name,
                parameter_types,
                return_types,
            );
        }
        self.state.library_function_ids =
            library_functions.iter().map(|f| f.node_id()).collect();

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
        let mut emitted_symbols: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for state_variable in contract.compute_linearised_state_variables() {
            let Some(&(slot, byte_offset, location)) =
                storage_layout.get(&state_variable.node_id())
            else {
                continue;
            };
            // Distinct state variables may share a symbol only through
            // inheritance re-linearisation; emit each once. A storage and a
            // transient variable may legitimately share (slot, offset) — the
            // location-aware symbol (e.g. `slot_0_0` vs `tslot_0_0`) keeps them
            // distinct.
            let symbol = Self::storage_symbol(slot, byte_offset, location);
            if !emitted_symbols.insert(symbol.clone()) {
                continue;
            }
            let element_type =
                TypeConversion::resolve_state_variable_type(&state_variable, &self.state.builder)?;
            self.state.builder.emit_sol_state_var(
                &symbol,
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

        // Emit the collected internal library functions into this contract's
        // body so the `sol.call`s above resolve.
        for library_function in &library_functions {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, contract, &storage_layout)
                .emit_sol(library_function, &contract_body)?;
            self.state.current_contract_type = None;
        }

        for state_variable in contract.compute_linearised_state_variables() {
            if matches!(state_variable.mutability(), StateVariableMutability::Constant) {
                self.emit_constant_getter(&state_variable, &contract_body)?;
            } else if let Some(&(slot, byte_offset, location)) =
                storage_layout.get(&state_variable.node_id())
            {
                self.emit_state_variable_getter(
                    &state_variable,
                    slot,
                    byte_offset,
                    location,
                    &contract_body,
                )?;
            }
        }

        Ok(())
    }

    /// Builds the storage-variable symbol name for a `(slot, byte_offset)`
    /// location. Packed small-value variables share a slot but differ in
    /// byte offset, so the offset is part of the symbol. Transient variables
    /// get a distinct `tslot_` prefix because transient and persistent storage
    /// number their slots independently and may otherwise collide.
    pub(crate) fn storage_symbol(
        slot: U256,
        byte_offset: u32,
        location: solx_utils::DataLocation,
    ) -> String {
        let prefix = if matches!(location, solx_utils::DataLocation::Transient) {
            "tslot"
        } else {
            "slot"
        };
        format!("{prefix}_{slot}_{byte_offset}")
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
        location: solx_utils::DataLocation,
        contract_body: &melior::ir::BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some(AbiEntry::Function(abi)) = state_variable.compute_abi_entry() else {
            return Ok(());
        };
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

        // Getter for a (possibly nested) value-result array / mapping: `m(K)`,
        // `a(uint256)`, `a(i, j)`, `m(k1, k2)`, mixed `m(k)[i]`, ... Each nesting
        // level chains a `sol.map` (mappings) or a bounds-checked `sol.gep`
        // (arrays) over its key/index argument; the final value is loaded.
        //
        // Array levels emit an explicit `index < length` check that **bare-reverts**
        // (`revert(0, 0)`) on out-of-bounds via a no-message `sol.require`, matching
        // solc's auto-generated accessor — NOT `sol.gep`'s `Panic(0x32)`, which the
        // semantic tests (expecting a bare `FAILURE`, i.e. empty revert data)
        // reject. Reference-typed keys or results are skipped (selector reverts).
        if !abi.inputs().is_empty() {
            enum GetterLevel<'c> {
                /// `sol.map` over a key; carries the mapped-slot reference type.
                Mapping(Type<'c>),
                /// Bounds-checked `sol.gep` over an index; carries the element type
                /// and, for fixed arrays, the static size (dynamic arrays: `None`).
                Array(Type<'c>, Option<u64>),
            }
            let mut input_types: Vec<Type<'context>> = Vec::new();
            let mut levels: Vec<GetterLevel<'context>> = Vec::new();
            let mut current = declared_type.clone();
            loop {
                match &current {
                    SlangType::Mapping(mapping_type) => {
                        let key_slang = mapping_type.key_type();
                        if key_slang.is_reference_type() {
                            return Ok(());
                        }
                        let value_slang = mapping_type.value_type();
                        let resolved_value = TypeConversion::resolve_slang_type(
                            &value_slang,
                            Some(location),
                            builder,
                        );
                        // Intermediate containers are addressed by their reference;
                        // a value terminal by a `!sol.ptr<V>`.
                        let level_type = if value_slang.is_reference_type() {
                            resolved_value
                        } else {
                            builder.types.pointer(resolved_value, location)
                        };
                        input_types.push(TypeConversion::resolve_slang_type(
                            &key_slang,
                            Some(location),
                            builder,
                        ));
                        levels.push(GetterLevel::Mapping(level_type));
                        current = value_slang;
                    }
                    SlangType::Array(array_type) => {
                        let element_slang = array_type.element_type();
                        let element_type = TypeConversion::resolve_slang_type(
                            &element_slang,
                            Some(location),
                            builder,
                        );
                        input_types.push(builder.types.ui256);
                        levels.push(GetterLevel::Array(element_type, None));
                        current = element_slang;
                    }
                    SlangType::FixedSizeArray(array_type) => {
                        let element_slang = array_type.element_type();
                        let element_type = TypeConversion::resolve_slang_type(
                            &element_slang,
                            Some(location),
                            builder,
                        );
                        input_types.push(builder.types.ui256);
                        levels.push(GetterLevel::Array(element_type, Some(array_type.size() as u64)));
                        current = element_slang;
                    }
                    _ => break,
                }
            }
            let result_slang = current;
            if input_types.is_empty()
                || input_types.len() != abi.inputs().len()
                || result_slang.is_reference_type()
            {
                return Ok(());
            }
            let container_type =
                TypeConversion::resolve_state_variable_type(state_variable, builder)?;
            let result_type =
                TypeConversion::resolve_slang_type(&result_slang, Some(location), builder);
            let entry = builder.emit_sol_func(
                &signature,
                &input_types,
                std::slice::from_ref(&result_type),
                Some(selector),
                StateMutability::View,
                None,
                contract_body,
            );
            let slot_name = Self::storage_symbol(slot, byte_offset, location);
            let mut base = builder.emit_sol_addr_of(&slot_name, container_type, &entry);
            for (index, level) in levels.iter().enumerate() {
                let arg: Value<'context, '_> = entry.argument(index)?.into();
                base = match level {
                    GetterLevel::Mapping(level_type) => {
                        builder.emit_sol_map(base, arg, *level_type, &entry)
                    }
                    GetterLevel::Array(element_type, fixed_size) => {
                        // Bounds-check `index < length`; OOB → bare `revert(0, 0)`.
                        let length = match fixed_size {
                            Some(size) => builder.emit_sol_constant(
                                *size as i64,
                                builder.types.ui256,
                                &entry,
                            ),
                            None => entry
                                .append_operation(
                                    solx_mlir::ods::sol::LengthOperation::builder(
                                        builder.context,
                                        builder.unknown_location,
                                    )
                                    .inp(base)
                                    .len(builder.types.ui256)
                                    .build()
                                    .into(),
                                )
                                .result(0)
                                .expect("sol.length produces one result")
                                .into(),
                        };
                        let in_bounds =
                            builder.emit_sol_cmp(arg, length, solx_mlir::CmpPredicate::Lt, &entry);
                        builder.emit_sol_require(in_bounds, None, &[], false, &entry);
                        builder.emit_sol_gep(base, arg, *element_type, &entry)
                    }
                };
            }
            let value = builder.emit_sol_load(base, result_type, &entry)?;
            builder.emit_sol_return(&[value], &entry);
            return Ok(());
        }

        let element_type = TypeConversion::resolve_state_variable_type(state_variable, builder)?;
        // A reference-typed state variable (`string`/`bytes`/array/struct) is
        // addressed by the reference type itself in storage; value types use
        // a `!sol.ptr<T, Storage>`. Matching the address type the initializer
        // uses keeps the `sol.addr_of` symbol consistent.
        let address_type = if declared_type.is_reference_type() {
            element_type
        } else {
            builder.types.pointer(element_type, location)
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
        let slot_name = Self::storage_symbol(slot, byte_offset, location);
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
    fn compute_storage_layout(contract: &ContractDefinition) -> StorageLayout {
        use solx_utils::DataLocation;
        let mut layout: StorageLayout = HashMap::new();
        if let Some(abi) = contract.compute_abi() {
            for item in abi.storage_layout().iter() {
                let byte_offset = u32::try_from(item.offset()).unwrap_or(0);
                layout.insert(item.node_id(), (item.slot(), byte_offset, DataLocation::Storage));
            }
            // Transient state variables (`T transient x;`) live in a separate
            // address space reached via TLOAD/TSTORE. Slang lays them out
            // independently of persistent storage, so a transient slot may
            // collide numerically with a storage slot; the `Transient` tag
            // routes every access through transient pointers/symbols.
            for item in abi.transient_storage_layout().iter() {
                let byte_offset = u32::try_from(item.offset()).unwrap_or(0);
                layout.insert(
                    item.node_id(),
                    (item.slot(), byte_offset, DataLocation::Transient),
                );
            }
        }
        // Slang's ABI omits `immutable` state variables (they live in code,
        // not storage). For the experimental Slang frontend we treat them as
        // ordinary storage variables so that compilation succeeds — runtime
        // behaviour around code immutability won't match solc's, but the
        // observable semantics (read after constructor write) survives.
        let mut next_slot: U256 = layout
            .values()
            .filter(|(_, _, location)| !matches!(location, DataLocation::Transient))
            .map(|(slot, _, _)| *slot)
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
            layout.insert(state_variable.node_id(), (next_slot, 0, DataLocation::Storage));
            next_slot += U256::from(1);
        }
        layout
    }
}
