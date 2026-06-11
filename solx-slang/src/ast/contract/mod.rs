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

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::LibraryDefinition;
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

        // Re-resolve `super.f(...)` / `Base.f(...)` against the C3 linearisation
        // (slang resolves them lexically, which is wrong in a diamond). The
        // redirect drives the call site; the shadowed base overrides reached
        // through `super` are emitted internal-only under contract-qualified
        // symbols below, and their bodies are walked by the free / library
        // collectors so the internals they call also register.
        let super_dispatch = super_call::SuperDispatch::build_super_dispatch(contract);
        self.state.super_redirect = super_dispatch.redirect.clone();
        self.state.virtual_redirect = super_dispatch.virtual_redirect.clone();
        let shadowed_functions: Vec<FunctionDefinition> = super_dispatch
            .shadowed
            .iter()
            .map(|(_, function)| function.clone())
            .collect();

        self.pre_register_functions(contract);
        // Register each shadowed base override under its contract-qualified
        // symbol so a `super`/`Base` call resolves to it by node id.
        for (symbol, function) in &super_dispatch.shadowed {
            let (parameter_types, return_types) =
                TypeConversion::resolve_function_types(function, &self.state.builder);
            self.state.register_function_signature(
                function.node_id(),
                symbol.clone(),
                parameter_types,
                return_types,
            );
        }

        // Free functions (`f(...)` declared at file level) reachable from this
        // contract, transitively. They are not in the linearised function set,
        // so pre-register their signatures here and emit their bodies into this
        // contract's module below, where they lower as ordinary internal
        // functions. (No `super`/library roots yet — those clusters extend the
        // `extra_roots` walk and add the duplicate-symbol filtering.)
        let mut reached_free_functions = FreeCallCollector::reachable_free_functions(
            contract,
            free_functions,
            &shadowed_functions,
        );

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
        let library_functions = LibraryCallCollector::reachable_library_functions(
            contract,
            free_functions,
            &shadowed_functions,
        );
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

        // Declare every state variable in the C3-linearised hierarchy (inherited
        // + own), not just this contract's own members: a derived contract owns
        // the FULL storage layout, and an inherited getter / inherited function
        // body emits `sol.addr_of @var` against an inherited slot, which the
        // backend's `AddrOfOpLowering` resolves by `lookupSymbol` in this
        // contract's module (asserts if the `sol.state_var` declaration is
        // absent).
        for state_variable in contract.linearised_state_variables() {
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
        FunctionEmitter::new(self.state, Some(contract), &storage_layout)
            .emit_constructor(&contract_body)?;
        self.state.current_contract_type = None;

        // An overridden public function whose signature matches an inherited
        // public state variable's auto-getter would emit a second function under
        // the getter's selector symbol (`redefinition of symbol`); the getter
        // (emitted last) wins, so skip such functions here.
        let getter_selectors: std::collections::HashSet<u32> = contract
            .linearised_state_variables()
            .iter()
            .filter_map(|state_variable| state_variable.compute_selector())
            .collect();

        // Walk the C3-linearised function set so a derived contract emits its
        // inherited methods (regular functions, fallback, receive) too — not just
        // its own — subject to override resolution. Constructors and modifiers are
        // emitted by their own paths (the constructor below; modifiers inline at
        // their call sites), so skip them here.
        // A contract has a single fallback and a single receive dispatcher
        // entry. The C3 linearisation lists the most-derived override first, so
        // emit the first fallback / receive encountered and skip any inherited
        // base versions — they have distinct signatures (`fallback(bytes)` vs an
        // overriding `fallback()`) so override resolution does not collapse them,
        // and emitting a second `sol.func` of either kind makes the backend
        // assert there is exactly one fallback / receive (`!fallbackFn`).
        let mut fallback_emitted = false;
        let mut receive_emitted = false;
        for function in contract.linearised_functions() {
            match function.kind() {
                FunctionKind::Constructor | FunctionKind::Modifier => continue,
                FunctionKind::Fallback if fallback_emitted => continue,
                FunctionKind::Fallback => fallback_emitted = true,
                FunctionKind::Receive if receive_emitted => continue,
                FunctionKind::Receive => receive_emitted = true,
                _ => {}
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
        // contract-qualified symbols (internal-only, no selector).
        for (symbol, function) in &super_dispatch.shadowed {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, Some(contract), &storage_layout)
                .emit_sol_with_symbol(function, symbol, &contract_body)?;
            self.state.current_contract_type = None;
        }

        // Emit the collected free functions so their `sol.call`s resolve. Each
        // is emitted under its node-id-qualified symbol so two file-level
        // functions of the same name and signature do not collide on one symbol.
        for free in &reached_free_functions {
            self.state.current_contract_type = Some(contract_type);
            FunctionEmitter::new(self.state, Some(contract), &storage_layout)
                .emit_sol_with_symbol(
                    free,
                    &Self::node_id_qualified_symbol(free),
                    &contract_body,
                )?;
            self.state.current_contract_type = None;
        }

        self.emit_state_variable_getters(contract, &storage_layout, &contract_body)?;

        Ok(())
    }

    /// Emits a deployable library object — its externally-dispatchable functions
    /// as `sol.func`s under a `sol.contract`, plus the method-identifier map.
    ///
    /// A `delegatecall`ed library object dispatches only its `external` /
    /// `public` functions; `internal` / `private` functions and modifiers are
    /// inlined into their callers, so they are not part of the library's own
    /// object. A library with no externally-visible function is therefore
    /// emitted as an empty, call-protected stub — matching solc, and avoiding
    /// standalone emission of inlined-only helpers that assume a caller context
    /// (e.g. a storage-parameter modifier), which would otherwise panic. The
    /// stub still exists in the build artifacts so the harness's `// library:`
    /// directive can deploy and link it.
    ///
    /// # Errors
    ///
    /// Returns an error if any function body contains unsupported constructs.
    pub fn emit_library(
        &mut self,
        library: &LibraryDefinition,
    ) -> anyhow::Result<(String, BTreeMap<String, String>)> {
        let library_name = library.name().name();

        let has_deployable_function = library.members().iter().any(|member| {
            matches!(
                member,
                ContractMember::FunctionDefinition(function)
                    if matches!(function.kind(), FunctionKind::Regular)
                        && matches!(
                            function.visibility(),
                            FunctionVisibility::External | FunctionVisibility::Public
                        )
            )
        });
        // When the library is deployable, emit all of its `Regular` functions
        // (the internal ones the dispatched functions call included); the
        // backend DCEs any left unreferenced. When it is not, emit none — the
        // empty stub.
        let functions: Vec<FunctionDefinition> = if has_deployable_function {
            library
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
                .collect()
        } else {
            Vec::new()
        };

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

        // A library has no state, so the storage layout is empty.
        let storage_layout: HashMap<NodeId, StorageSlot> = HashMap::new();
        let library_type = self.state.builder.types.contract(&library_name, false);
        let module_body = self.state.module.body();
        let contract_body = self.state.builder.emit_sol_contract(
            &library_name,
            // A library is `ContractKind::Library`: the backend dispatcher passes
            // a `storage` reference parameter as its slot (instead of ABI-decoding
            // it) and emits the library-address self-reference as a
            // `llvm.setimmutable`. That immutable is lowered to a heap store in the
            // deploy segment's MLIR→LLVM step (`translate_source_to_llvm`'s
            // `lowerSetImmutables`), using the offsets the runtime object reserves.
            solx_mlir::ContractKind::Library,
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
        for state_variable in contract.linearised_state_variables() {
            if matches!(
                state_variable.mutability(),
                StateVariableMutability::Constant
            ) {
                self.emit_constant_getter(&state_variable, storage_layout, contract_body)?;
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
    /// are emitted. Walks the C3-linearised function set (override-resolved, one
    /// entry per signature) so an inherited method called by its bare name in a
    /// derived contract resolves to its registered symbol — not only the
    /// contract's own functions, which would leave every inherited call
    /// unresolved.
    fn pre_register_functions(&mut self, contract: &ContractDefinition) {
        for function in contract.linearised_functions() {
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
