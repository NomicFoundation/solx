//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Function definition lowering to Sol dialect MLIR.
pub mod function;
mod free_function;
mod library;
mod super_call;

use std::collections::BTreeMap;
use std::collections::HashMap;

use ruint::aliases::U256;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;

use melior::ir::BlockLike;
use melior::ir::Type;
use melior::ir::r#type::TypeLike;
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
    pub fn emit(
        &mut self,
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
    ) -> anyhow::Result<()> {
        let contract_name = contract.name().name();

        // Re-resolve `super` calls against the C3 linearisation (slang resolves
        // them lexically, which is wrong in a diamond). The redirect drives the
        // call site; the shadowed base overrides are emitted internal-only under
        // contract-qualified symbols (below).
        let super_dispatch = super_call::build_super_dispatch(contract);
        self.state.super_redirect = super_dispatch.redirect.clone();
        self.state.virtual_redirect = super_dispatch.virtual_redirect.clone();

        self.pre_register_functions(contract, &super_dispatch.shadowed);

        // Shadowed base overrides reached through `super` are emitted into this
        // module (below) but are not in the linearised set, so the library- and
        // free-function collectors must also walk their bodies to register the
        // internals they call.
        let shadowed_functions: Vec<FunctionDefinition> = super_dispatch
            .shadowed
            .iter()
            .map(|(_, function)| function.clone())
            .collect();

        // Internal library functions called by the contract (`L.f(...)`) are
        // not part of `compute_linearised_functions`, so pre-register them and
        // emit their bodies into this contract's module below — they lower like
        // ordinary internal functions.
        let library_functions =
            library::collect_library_functions(contract, free_functions, &shadowed_functions);
        for library_function in &library_functions {
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(library_function, &self.state.builder);
            self.state.register_function_signature(
                library_function.node_id(),
                Self::library_function_symbol(library_function),
                parameter_types,
                return_types,
            );
        }
        self.state.library_function_ids =
            library_functions.iter().map(|f| f.node_id()).collect();

        // Free functions (`f(...)` declared at file level) called by the
        // contract, transitively. Like library internals, they are not in the
        // linearised set, so pre-register and emit them into this module where
        // they lower as ordinary internal functions.
        //
        // A free function reached through an import-namespace member access
        // (`import "a.sol" as M; M.f(...)`) is also collected by the library
        // collector (its operand is an import qualifier, not a contract). When
        // the same function is *also* called bare (`f(...)`), it lands in both
        // sets; emitting it on both paths would now collide, since the library
        // and free symbols are both the node-id-qualified form. The library
        // emission already covers it (same symbol, same body), so drop the
        // duplicate here.
        let reached_free_functions: Vec<FunctionDefinition> =
            free_function::collect_free_functions(contract, free_functions, &shadowed_functions)
                .into_iter()
                .filter(|free| !self.state.library_function_ids.contains(&free.node_id()))
                .collect();
        for free in &reached_free_functions {
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(free, &self.state.builder);
            self.state.register_function_signature(
                free.node_id(),
                Self::free_function_symbol(free),
                parameter_types,
                return_types,
            );
        }

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
        FunctionEmitter::new(self.state, Some(contract), &storage_layout)
            .emit_constructor(&contract_body)?;
        self.state.current_contract_type = None;

        // A `public` state variable may `override` an inherited function: its
        // auto-generated getter takes over the function's selector (and ABI
        // entry). Slang still lists the overridden base function, which would
        // emit a second function under the getter's `selector()` symbol
        // (`redefinition of symbol`). Collect the getter selectors so the
        // overridden function is skipped below; the getter (emitted later) wins.
        let getter_selectors: std::collections::HashSet<u32> = contract
            .compute_linearised_state_variables()
            .iter()
            .filter_map(|state_variable| state_variable.compute_selector())
            .collect();

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
            if let Some(selector) = function.compute_selector()
                && getter_selectors.contains(&selector)
            {
                continue;
            }
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, Some(contract), &storage_layout)
                .emit_sol(&function, &contract_body)?;
            self.state.current_contract_type = None;
        }

        // Emit shadowed base overrides reached through `super` under their
        // contract-qualified, selector-less symbols. Bounded to functions a
        // `super` call actually targets, so contracts that don't use `super`
        // (or only reach non-overridden inherited functions) emit nothing extra.
        for (symbol, function) in &super_dispatch.shadowed {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, Some(contract), &storage_layout)
                .emit_sol_with_symbol(function, symbol, &contract_body)?;
            self.state.current_contract_type = None;
        }

        // Emit the collected internal library functions into this contract's
        // body so the `sol.call`s above resolve.
        for library_function in &library_functions {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, Some(contract), &storage_layout)
                .emit_sol_with_symbol(
                    library_function,
                    &Self::library_function_symbol(library_function),
                    &contract_body,
                )?;
            self.state.current_contract_type = None;
        }

        // Emit the collected free functions so their `sol.call`s resolve. Each
        // is emitted under its node-id-qualified symbol (see
        // `free_function_symbol`) so two file-level functions of the same name
        // and signature — reachable together when one is imported under an
        // alias — do not collide on a single MLIR symbol.
        for free in &reached_free_functions {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, Some(contract), &storage_layout)
                .emit_sol_with_symbol(free, &Self::free_function_symbol(free), &contract_body)?;
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

    /// Emits a `sol.contract` (kind `Library`) for a library definition so it
    /// can be deployed and `delegatecall`ed by `L.f(...)` callers. Libraries
    /// have no inheritance, state, or constructor, so this is a flat emission of
    /// the library's functions — `external`/`public` ones get selectors (and a
    /// dispatcher), `internal` ones lower as ordinary functions reachable from
    /// them. Returns the library name and its external method-identifier table.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub fn emit_library(
        &mut self,
        library: &LibraryDefinition,
    ) -> anyhow::Result<(String, BTreeMap<String, String>)> {
        let library_name = library.name().name();
        let functions: Vec<FunctionDefinition> = library
            .members()
            .iter()
            .filter_map(|member| match member {
                ContractMember::FunctionDefinition(function)
                    if matches!(function.kind(), FunctionKind::Regular) =>
                {
                    Some(function)
                }
                _ => None,
            })
            .collect();

        // Pre-register every function so calls between the library's functions
        // resolve before any body is emitted.
        for function in &functions {
            let mlir_name = FunctionEmitter::mlir_function_name(function);
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(function, &self.state.builder);
            self.state.register_function_signature(
                function.node_id(),
                mlir_name,
                parameter_types,
                return_types,
            );
        }

        let storage_layout = StorageLayout::new();
        let library_type = self.state.builder.types.contract(&library_name, false);
        let module_body = self.state.module.body();
        let contract_body = self.state.builder.emit_sol_contract(
            &library_name,
            // Emit as a plain contract: the `Library` kind triggers the
            // library-address self-reference (an immutable) the slang path does
            // not set up. A `delegatecall`ed library object only needs the
            // external-function dispatcher, which the contract kind provides.
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        for function in &functions {
            self.state.current_contract_type = Some(library_type);
            FunctionEmitter::new(self.state, None, &storage_layout)
                .emit_sol(function, &contract_body)?;
            self.state.current_contract_type = None;
        }

        let mut method_identifiers = BTreeMap::new();
        for function in &functions {
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }

        Ok((library_name, method_identifiers))
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
                        // A reference-typed key (`string` / `bytes`) is an ABI
                        // input decoded into memory. slang reports the key with
                        // the mapping's storage location, so build the memory
                        // string/bytes type directly rather than resolving it
                        // (which would yield a *storage* string the ABI decoder
                        // can't produce — it would hash the calldata offset, not
                        // the key bytes). `sol.map` hashes the key bytes for the
                        // slot, matching the constructor's `x["abc"]` write.
                        let key_type = if key_slang.is_reference_type() {
                            builder.types.string(solx_utils::DataLocation::Memory)
                        } else {
                            TypeConversion::resolve_slang_type(&key_slang, Some(location), builder)
                        };
                        input_types.push(key_type);
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
            if input_types.is_empty() || input_types.len() != abi.inputs().len() {
                return Ok(());
            }
            let container_type =
                TypeConversion::resolve_state_variable_type(state_variable, builder)?;
            let result_type =
                TypeConversion::resolve_slang_type(&result_slang, Some(location), builder);
            // A struct result expands into a tuple of its value-type members,
            // skipping mapping/array/nested-struct members — matching solc's
            // auto-generated accessor (slang's `can_return_from_getter`). Only
            // all-value structs are handled here; a returnable string/bytes/function
            // member needs storage→memory handling we don't do yet, so such a getter
            // is skipped (left ungenerated) rather than emitted incorrectly.
            // Each plan entry is `(member_index, gep_type, result_type)`. For a
            // value member the two types are identical (`sol.gep` yields a
            // pointer, `sol.load` reads the value). For a `string`/`bytes` member
            // the gep yields the storage reference and the result is a *memory*
            // string, so `sol.load` copies storage→memory — a multi-value return
            // ABI-encodes a memory string correctly, whereas a raw storage
            // reference would be mis-wrapped.
            let struct_plan: Option<Vec<(u64, Type<'context>, Type<'context>)>> = match &result_slang
            {
                SlangType::Struct(struct_type) => {
                    let Definition::Struct(struct_definition) = struct_type.definition() else {
                        return Ok(());
                    };
                    match Self::struct_getter_plan(&struct_definition, result_type, builder) {
                        Some(plan) => Some(plan),
                        None => return Ok(()),
                    }
                }
                // Other reference results (`string`/`bytes`/array) aren't handled yet.
                _ if result_slang.is_reference_type() => return Ok(()),
                _ => None,
            };
            let result_types: Vec<Type<'context>> = match &struct_plan {
                Some(plan) => plan.iter().map(|(_, _, result_type)| *result_type).collect(),
                None => vec![result_type],
            };
            let entry = builder.emit_sol_func(
                &signature,
                &input_types,
                &result_types,
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
            match &struct_plan {
                Some(plan) => {
                    // Expand the struct at `base` into its returnable-member tuple.
                    let mut values = Vec::new();
                    for (member_index, member_type, result_member_type) in plan {
                        let index_value = builder.emit_sol_constant(
                            *member_index as i64,
                            builder.types.ui64,
                            &entry,
                        );
                        let address =
                            builder.emit_sol_gep(base, index_value, *member_type, &entry);
                        values.push(Self::read_getter_member(
                            builder,
                            address,
                            *member_type,
                            *result_member_type,
                            &entry,
                        )?);
                    }
                    builder.emit_sol_return(&values, &entry);
                }
                None => {
                    let value = builder.emit_sol_load(base, result_type, &entry)?;
                    builder.emit_sol_return(&[value], &entry);
                }
            }
            return Ok(());
        }

        // A no-argument getter for a `public` struct state variable returns the
        // struct's value/string/bytes members as a FLATTENED tuple (omitting
        // mapping/array/nested-struct members), matching solc — not the struct
        // as a single value (which would ABI-encode with a spurious outer tuple
        // offset). Mapping/array struct getters are handled in the input block
        // above; this is the bare `S public x` case (no getter arguments).
        if let SlangType::Struct(struct_type) = &declared_type
            && let Definition::Struct(struct_definition) = struct_type.definition()
        {
            let struct_mlir_type =
                TypeConversion::resolve_slang_type(&declared_type, Some(location), builder);
            if let Some(plan) =
                Self::struct_getter_plan(&struct_definition, struct_mlir_type, builder)
            {
                let result_types: Vec<Type<'context>> =
                    plan.iter().map(|(_, _, result_type)| *result_type).collect();
                let container_type =
                    TypeConversion::resolve_state_variable_type(state_variable, builder)?;
                let entry = builder.emit_sol_func(
                    &signature,
                    &[],
                    &result_types,
                    Some(selector),
                    StateMutability::View,
                    None,
                    contract_body,
                );
                let slot_name = Self::storage_symbol(slot, byte_offset, location);
                let base = builder.emit_sol_addr_of(&slot_name, container_type, &entry);
                let mut values = Vec::new();
                for (member_index, member_type, result_member_type) in &plan {
                    let index_value = builder.emit_sol_constant(
                        *member_index as i64,
                        builder.types.ui64,
                        &entry,
                    );
                    let address = builder.emit_sol_gep(base, index_value, *member_type, &entry);
                    values.push(Self::read_getter_member(
                        builder,
                        address,
                        *member_type,
                        *result_member_type,
                        &entry,
                    )?);
                }
                builder.emit_sol_return(&values, &entry);
                return Ok(());
            }
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

    /// Builds the member-expansion plan for a `public` struct getter, mirroring
    /// solc's auto-accessor: value and `string`/`bytes` members are returned as
    /// a flattened tuple; mapping / array / nested-struct members are omitted.
    ///
    /// Each entry is `(member_index, gep_type, result_type)`. For a value member
    /// the two types match (gep → pointer, load → value). For a `string`/`bytes`
    /// member the gep yields the storage reference and the result is a *memory*
    /// string, so `sol.load` copies storage→memory (a multi-value return
    /// ABI-encodes a memory string correctly; a raw storage reference is
    /// mis-wrapped). Returns `None` when the getter cannot be generated — a
    /// member with no resolved type, a non-string/bytes reference (function
    /// pointer / unexpanded aggregate), or an empty plan.
    ///
    /// `struct_mlir_type` is the resolved (storage-located) struct type, whose
    /// element types `mlirSolGetEltType` indexes by AST member position.
    fn struct_getter_plan(
        struct_definition: &slang_solidity_v2::ast::StructDefinition,
        struct_mlir_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>> {
        let mut plan = Vec::new();
        for (member_index, member) in struct_definition.members().iter().enumerate() {
            let is_string_or_bytes = match member.get_type() {
                Some(
                    SlangType::Mapping(_)
                    | SlangType::Array(_)
                    | SlangType::FixedSizeArray(_)
                    | SlangType::Struct(_),
                ) => continue,
                Some(SlangType::String(_) | SlangType::Bytes(_)) => true,
                Some(_) => false,
                None => return None,
            };
            // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
            // `sol::getEltType` for the struct field at `member_index` (which
            // mirrors the AST member index, including skipped members, exactly
            // as `emit_struct_field_address` does).
            let member_type = unsafe {
                Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                    struct_mlir_type.to_raw(),
                    member_index as u64,
                ))
            };
            let result_member_type = if is_string_or_bytes {
                builder.types.string(solx_utils::DataLocation::Memory)
            } else if solx_mlir::TypeFactory::is_sol_reference(member_type)
                || solx_mlir::TypeFactory::is_sol_function_ref(member_type)
            {
                return None;
            } else {
                member_type
            };
            plan.push((member_index as u64, member_type, result_member_type));
        }
        if plan.is_empty() {
            return None;
        }
        Some(plan)
    }

    /// Reads one struct-getter member from its in-storage address (the `sol.gep`
    /// result) into the value returned to the ABI encoder.
    ///
    /// A value member (`member_type == result_member_type`) loads its value. A
    /// `string`/`bytes` member's gep yields the storage reference, which is
    /// converted to the memory result type — a storage→memory copy, exactly as
    /// returning a storage `string` as `string memory` does (`emit_return`).
    fn read_getter_member<'block>(
        builder: &solx_mlir::Builder<'context>,
        address: Value<'context, 'block>,
        member_type: Type<'context>,
        result_member_type: Type<'context>,
        block: &melior::ir::BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        if member_type == result_member_type {
            builder.emit_sol_load(address, result_member_type, block)
        } else {
            Ok(TypeConversion::from_target_type(result_member_type, builder)
                .emit(address, builder, block))
        }
    }

    /// The symbol under which a collected internal library function is emitted
    /// into the calling contract's body.
    ///
    /// It must not collide with a contract function — or another library's
    /// function — of the same signature (`redefinition of symbol`), e.g. a
    /// `library L { function f() internal ... }` reached from a contract that
    /// also defines `f`. Qualify it with the function's globally-unique node id.
    /// Library internals have no selector and are only ever resolved by node id
    /// (`resolve_function` via `library_function_ids`), so the exact spelling is
    /// immaterial — it just has to be unique.
    fn library_function_symbol(function: &FunctionDefinition) -> String {
        format!(
            "{}#{:?}",
            FunctionEmitter::mlir_function_name(function),
            function.node_id()
        )
    }

    /// The MLIR symbol for a file-level free function emitted into a contract
    /// module. Two distinct free functions may share a name and signature when
    /// one is imported under an alias (`import {f as g} from "..."`, or
    /// `import "..." as M` exposing `M.f` alongside a local `f`) and both are
    /// reachable from the same contract — e.g. one from a derived function and
    /// the other from a base override reached via `super`. Free functions have
    /// no selector and are only ever resolved by node id (`resolve_function`),
    /// so qualifying the symbol with the globally-unique node id keeps the two
    /// distinct without affecting call resolution.
    fn free_function_symbol(function: &FunctionDefinition) -> String {
        format!(
            "{}#{:?}",
            FunctionEmitter::mlir_function_name(function),
            function.node_id()
        )
    }

    /// Pre-registers all function signatures for call resolution before bodies
    /// are emitted. `shadowed` are the base overrides reached through `super`,
    /// registered under contract-qualified symbols so `super` calls resolve to
    /// a distinct internal function from the most-derived version.
    fn pre_register_functions(
        &mut self,
        contract: &ContractDefinition,
        shadowed: &[(String, FunctionDefinition)],
    ) {
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

        for (symbol, function) in shadowed {
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(function, &self.state.builder);
            self.state.register_function_signature(
                function.node_id(),
                symbol.clone(),
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
