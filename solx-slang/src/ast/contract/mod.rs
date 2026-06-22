//!
//! Contract / library definition emission to Sol dialect MLIR.
//!

pub mod analysis;
pub mod constructor;
pub mod function;
pub mod getter;
pub mod object_scope;
pub mod storage_layout;

pub use self::object_scope::ObjectScope;

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::StateVarOperation;
use solx_utils::DataLocation;

use self::analysis::free_function::FreeCallCollector;
use self::analysis::library::LibraryCallCollector;
use self::function::FunctionScope;
use self::function::expression::ExpressionContext;
use self::function::expression::arithmetic_mode::ArithmeticMode;
use self::function::mlir_symbol_name::MlirSymbolName;
use self::storage_layout::StorageSlot;
use crate::ast::EmitExpression;
use crate::ast::EmitFunction;
use crate::ast::EmitObject;
use crate::ast::Type as AstType;
use crate::ast::emit::EmitConstructor;
use crate::ast::pending_queries::StorageLayout;

impl EmitObject for ContractDefinition {
    fn emit(&self, context: &mut Context, scope: &ObjectScope) {
        let contract_name = self.name().name();

        // Re-resolve `super.f(...)` / `Base.f(...)` against the C3 linearisation (slang resolves them
        // lexically, wrong in a diamond). Shadowed base overrides are emitted internal-only below.
        let super_dispatch = self::analysis::super_call::SuperDispatch::build_super_dispatch(self);
        context.super_redirect = super_dispatch.redirect.clone();
        context.virtual_redirect = super_dispatch.virtual_redirect.clone();
        let shadowed_functions: Vec<_> = super_dispatch
            .shadowed
            .iter()
            .map(|(_, function)| function.clone())
            .collect();

        // Pre-register the C3-linearised function set (override-resolved) so an inherited method
        // called by bare name in a derived contract resolves to its registered symbol.
        self.register_signatures(
            context,
            self.linearised_functions()
                .into_iter()
                .filter(|function| !matches!(function.kind(), FunctionKind::Modifier))
                .map(|function| {
                    let symbol = function.mlir_function_name();
                    (function, symbol)
                }),
        );
        // Register each shadowed base override under its contract-qualified
        // symbol so a `super`/`Base` call resolves to it by node id.
        self.register_signatures(
            context,
            super_dispatch
                .shadowed
                .iter()
                .map(|(symbol, function)| (function.clone(), symbol.clone())),
        );

        // Free functions reachable from this contract, transitively — not in the linearised set,
        // so registered here and emitted as ordinary internal functions below.
        let mut reached_free_functions = FreeCallCollector::reachable_free_functions(
            self,
            scope.free_functions,
            &shadowed_functions,
        );

        // Out-of-band sources the reachability walk misses by name, deduped against one growing `seen`
        // set: operator-bound functions and internal library functions (`L.f(...)`).
        let mut seen: HashSet<NodeId> = reached_free_functions
            .iter()
            .map(|function| function.node_id())
            .collect();
        let library_functions = LibraryCallCollector::reachable_library_functions(
            self,
            scope.free_functions,
            &shadowed_functions,
        );
        for function in scope
            .operator_functions
            .iter()
            .cloned()
            .chain(library_functions)
        {
            if seen.insert(function.node_id()) {
                reached_free_functions.push(function);
            }
        }

        // Register each reached free function under its node-id-qualified symbol so calls resolve
        // regardless of emission order, even for same-named functions.
        self.register_signatures(
            context,
            reached_free_functions
                .iter()
                .map(|function| (function.clone(), function.node_id_qualified_symbol())),
        );

        let storage_layout = self.storage_layout();
        let contract_type =
            AstType::contract(context.builder.context, &contract_name, self.is_payable())
                .into_mlir();

        let module_body = context.module.body();
        let contract_body = self.emit_contract_shell(
            context,
            &contract_name,
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        // Declare every state variable in the C3-linearised hierarchy (inherited + own): a derived
        // contract owns the FULL layout, and inherited getters / bodies address inherited slots by symbol.
        for state_variable in self.linearised_state_variables() {
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let element_type = AstType::resolve_state_variable(
                &state_variable.get_type().expect("slang validated"),
                &context.builder,
            );
            let builder = &context.builder;
            let slot_attribute: IntegerAttribute =
                Attribute::parse(builder.context, &format!("{} : i256", slot.slot))
                    .expect("valid slot literal")
                    .try_into()
                    .expect("slot literal is an integer attribute");
            let byte_offset_attribute = IntegerAttribute::new(
                IntegerType::new(builder.context, solx_utils::BIT_LENGTH_X32 as u32).into(),
                slot.byte_offset.into(),
            );
            let mut operation =
                StateVarOperation::builder(builder.context, builder.unknown_location)
                    .sym_name(StringAttribute::new(builder.context, &slot.name))
                    .r#type(TypeAttribute::new(element_type))
                    .slot(slot_attribute)
                    .byte_offset(byte_offset_attribute);
            // A `transient` variable (EIP-1153) lives in the separate transient slot
            // space; the attribute makes its accesses lower to TLOAD/TSTORE.
            if matches!(slot.location, DataLocation::Transient) {
                operation = operation.transient(Attribute::unit(builder.context));
            }
            contract_body.append_operation(operation.build().into());
        }

        context.current_contract_type = Some(contract_type);
        self.emit_constructor(
            &FunctionScope::new(context, Some(self), &storage_layout),
            &contract_body,
        );
        context.current_contract_type = None;

        // An overridden public function matching an inherited public state variable's auto-getter
        // would collide on the getter's selector symbol; the getter (emitted last) wins, so skip it here.
        let getter_selectors: HashSet<u32> = self
            .linearised_state_variables()
            .iter()
            .filter_map(|state_variable| state_variable.compute_selector())
            .collect();

        // Walk the C3-linearised function set so a derived contract emits inherited methods too
        // (override-resolved). The most-derived override is first; emit the first fallback / receive only.
        let mut fallback_emitted = false;
        let mut receive_emitted = false;
        for function in self.linearised_functions() {
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
            context.current_contract_type = Some(contract_type);
            function.emit(
                &FunctionScope::new(context, Some(self), &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }

        // Emit shadowed base overrides reached through `super` under their
        // contract-qualified symbols (internal-only, no selector).
        for (symbol, function) in &super_dispatch.shadowed {
            context.current_contract_type = Some(contract_type);
            function.emit_with_symbol(
                &FunctionScope::new(context, Some(self), &storage_layout),
                symbol,
                &contract_body,
            );
            context.current_contract_type = None;
        }

        // Emit the collected free functions, each under its node-id-qualified
        // symbol so two file-level functions of the same signature do not collide.
        for free in reached_free_functions.iter() {
            context.current_contract_type = Some(contract_type);
            free.emit_with_symbol(
                &FunctionScope::new(context, Some(self), &storage_layout),
                &free.node_id_qualified_symbol(),
                &contract_body,
            );
            context.current_contract_type = None;
        }

        // Auto-generated external accessors for `public` state variables; each
        // reads its slot over the shared (empty) emission scope.
        let environment = Environment::new();
        let getter_context = ExpressionContext::new(
            context,
            &environment,
            &storage_layout,
            ArithmeticMode::Checked,
        );
        for state_variable in self.linearised_state_variables() {
            state_variable.emit(&getter_context, contract_body);
        }
    }
}

impl EmitObject for LibraryDefinition {
    /// Emits a deployable library object — its `external` / `public` functions as `sol.func`s.
    ///
    /// A library with no externally-visible function is emitted as an empty, call-protected stub
    /// (matching solc), kept in the artifacts so the `// library:` directive can deploy and link it.
    fn emit(&self, context: &mut Context, _scope: &ObjectScope) {
        let library_name = self.name().name();

        let has_deployable_function = self.members().iter().any(|member| {
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
        // A deployable library emits all `Regular` functions (the backend DCEs unreferenced ones);
        // a non-deployable one emits none — the empty stub.
        let functions: Vec<_> = if has_deployable_function {
            self.members()
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
        self.register_signatures(
            context,
            functions
                .iter()
                .map(|function| (function.clone(), function.mlir_function_name())),
        );

        // A library has no state, so the storage layout is empty.
        let storage_layout: HashMap<NodeId, StorageSlot> = HashMap::new();
        let library_type =
            AstType::contract(context.builder.context, &library_name, false).into_mlir();
        let module_body = context.module.body();
        // A library is `ContractKind::Library`: the dispatcher passes a `storage` reference parameter
        // as its slot, and the library-address self-reference lowers to `llvm.setimmutable`.
        let contract_body = self.emit_contract_shell(
            context,
            &library_name,
            solx_mlir::ContractKind::Library,
            &module_body,
        );

        for function in functions.iter() {
            context.current_contract_type = Some(library_type);
            function.emit(
                &FunctionScope::new(context, None, &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }
    }
}
