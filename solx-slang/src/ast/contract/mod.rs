//!
//! Contract definition lowering to Sol dialect MLIR.
//!

pub mod free_function;
/// Function definition lowering to Sol dialect MLIR.
pub mod function;
pub mod getter;
pub mod getter_level;
pub mod library;
pub mod reachability;
/// Contract storage layout: the slot assignment of state variables.
pub mod storage_layout;
pub mod super_call;

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Context;
use solx_utils::DataLocation;

use self::free_function::FreeCallCollector;
use self::function::FunctionEmitter;
use self::library::LibraryCallCollector;
use self::storage_layout::StorageSlot;
use crate::ast::operator_binding::OperatorBindings;
use crate::ast::type_conversion::TypeConversion;

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
    pub fn emit(
        &mut self,
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
        operator_bindings: &OperatorBindings,
    ) -> anyhow::Result<()> {
        let contract_name = contract.name().name();
        self.state.operator_bindings = operator_bindings.map.clone();

        self.pre_register_functions(contract);

        // Free functions (`f(...)` declared at file level) reachable from this
        // contract, transitively. They are not in the linearised function set,
        // so pre-register their signatures here and emit their bodies into this
        // contract's module below, where they lower as ordinary internal
        // functions. (No `super`/library roots yet — those clusters extend the
        // `extra_roots` walk and add the duplicate-symbol filtering.)
        let mut reached_free_functions =
            FreeCallCollector::reachable_free_functions(contract, free_functions, &[]);

        // Out-of-band function sources the reachability walk does not reach by
        // name are appended through one growing `seen` set, so each dedup is
        // against everything appended so far (not a per-source stale snapshot).
        let mut seen: HashSet<NodeId> = reached_free_functions
            .iter()
            .map(|function| function.node_id())
            .collect();

        // Operator functions bound via `using {f as op} for T global;` are free
        // functions the reachability walk misses — they are never called by
        // name, only invoked through an operator (`a + b`). They lower as ordinary
        // internal functions and the backend drops any left unused.
        Self::extend_unreached(
            &mut reached_free_functions,
            &mut seen,
            operator_bindings.functions.iter().cloned(),
        );

        // Internal (no-selector) library functions called via `L.f(...)` or
        // using-for `x.f(...)` are not in the linearised set either; append the
        // not-already-reached ones so they register and emit as ordinary internal
        // `sol.func`s (a library call resolves them by node id).
        let library_functions =
            LibraryCallCollector::reachable_library_functions(contract, free_functions, &[]);
        Self::extend_unreached(&mut reached_free_functions, &mut seen, library_functions);

        self.register_function_signatures(&reached_free_functions);

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
                matches!(slot.location, DataLocation::Transient),
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

        // Emit the collected free functions so their `sol.call`s resolve. Each
        // is emitted under its node-id-qualified symbol so two file-level
        // functions of the same name and signature do not collide on one symbol.
        for free in &reached_free_functions {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, contract, &storage_layout).emit_sol_with_symbol(
                free,
                &Self::node_id_qualified_symbol(free),
                &contract_body,
            )?;
            self.state.current_contract_type = None;
        }

        self.emit_state_variable_getters(contract, &storage_layout, &contract_body)?;

        Ok(())
    }

    /// Synthesises the auto-generated external accessor for each `public` state
    /// variable: `constant` variables fold to a pure literal getter, scalar
    /// storage variables read their slot. Struct and indexed (mapping/array)
    /// getters resolve through the same dispatcher; a variable whose accessor is
    /// not yet supported is left ungenerated (the rest of the contract still
    /// compiles).
    fn emit_state_variable_getters(
        &self,
        contract: &ContractDefinition,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        for state_variable in contract.compute_linearised_state_variables() {
            if matches!(
                state_variable.mutability(),
                StateVariableMutability::Constant
            ) {
                self.emit_constant_getter(&state_variable, contract_body)?;
            } else if let Some(slot) = storage_layout.get(&state_variable.node_id()) {
                self.emit_state_variable_getter(
                    &state_variable,
                    slot,
                    slot.location,
                    contract_body,
                )?;
            }
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

    /// Appends to `reached` every function in `additions` whose node id is not
    /// already in `seen`, growing `seen` as it goes — the single dedup-append for
    /// the out-of-band function sources (operator-bound, library) that the
    /// reachability walk does not reach by name. One growing set keeps each
    /// source deduplicated against all those appended before it.
    fn extend_unreached(
        reached: &mut Vec<FunctionDefinition>,
        seen: &mut HashSet<NodeId>,
        additions: impl IntoIterator<Item = FunctionDefinition>,
    ) {
        for function in additions {
            if seen.insert(function.node_id()) {
                reached.push(function);
            }
        }
    }

    /// Registers each free function's `(symbol, parameter types, return types)`
    /// signature under its node-id-qualified symbol, so calls to it resolve to a
    /// distinct internal `sol.func` even when a same-named function is reached
    /// together. Pre-registration runs before any body is emitted so calls
    /// resolve regardless of emission order.
    fn register_function_signatures(&mut self, functions: &[FunctionDefinition]) {
        for function in functions {
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(function, &self.state.builder);
            self.state.register_function_signature(
                function.node_id(),
                Self::node_id_qualified_symbol(function),
                parameter_types,
                return_types,
            );
        }
    }

    /// A function's MLIR symbol qualified by its globally-unique node id, so two
    /// file-level functions of the same canonical signature — reachable together
    /// when one is imported under an alias — do not collide on a single symbol.
    /// These functions are only ever resolved by node id, so the exact spelling
    /// is immaterial.
    fn node_id_qualified_symbol(function: &FunctionDefinition) -> String {
        format!(
            "{}#{:?}",
            FunctionEmitter::mlir_function_name(function),
            function.node_id()
        )
    }
}
