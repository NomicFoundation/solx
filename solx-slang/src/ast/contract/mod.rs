//!
//! Contract / library definition emission to Sol dialect MLIR.
//!

pub mod constructor;
pub mod contract_dispatch;
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
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::ImmutableOperation;
use solx_mlir::ods::sol::StateVarOperation;
use solx_utils::DataLocation;

use self::contract_dispatch::ContractDispatch;
use self::function::FunctionScope;
use self::function::expression::ExpressionContext;
use self::function::expression::arithmetic_mode::ArithmeticMode;
use self::function::mlir_symbol_name::MlirSymbolName;
use self::storage_layout::StorageSlot;
use crate::ast::EmitExpression;
use crate::ast::EmitFunction;
use crate::ast::EmitObject;
use crate::ast::Type as AstType;
use crate::ast::analysis::query::StorageLayout;
use crate::ast::analysis::walk::free_function::FreeCallCollector;
use crate::ast::analysis::walk::library::LibraryCallCollector;
use crate::ast::emit::EmitConstructor;
use crate::ast::emit::EmitModifierCalls;

impl EmitObject for ContractDefinition {
    fn emit(&self, context: &mut Context, scope: &ObjectScope) {
        let contract_name = self.name().name();

        let super_dispatch =
            crate::ast::analysis::walk::super_call::SuperDispatch::build_super_dispatch(self);
        let dispatch = ContractDispatch::from_super_dispatch(&super_dispatch);
        let shadowed_functions: Vec<_> = super_dispatch
            .shadowed
            .iter()
            .map(|(_, function)| function.clone())
            .collect();

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
        self.register_signatures(
            context,
            super_dispatch
                .shadowed
                .iter()
                .map(|(symbol, function)| (function.clone(), symbol.clone())),
        );

        // Both reachability walks are seeded with the operator-bound functions too: a function
        // reached ONLY transitively through a user-defined operator would otherwise be missed by
        // the by-name walk and panic at emission.
        let mut walk_roots = shadowed_functions;
        walk_roots.extend(scope.operator_functions.iter().cloned());

        let library_functions = LibraryCallCollector::reachable_library_functions(
            self,
            scope.free_functions,
            &walk_roots,
        );
        walk_roots.extend(library_functions.iter().cloned());

        let mut reached_free_functions =
            FreeCallCollector::reachable_free_functions(self, scope.free_functions, &walk_roots);

        let mut seen: HashSet<NodeId> = reached_free_functions
            .iter()
            .map(|function| function.node_id())
            .collect();
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

        self.register_signatures(
            context,
            reached_free_functions
                .iter()
                .map(|function| (function.clone(), function.node_id_qualified_symbol())),
        );

        let storage_layout = self.storage_layout();
        let contract_type =
            AstType::contract(context.mlir(), &contract_name, self.is_payable()).into_mlir();

        let module_body = context.module.body();
        let contract_body = self.emit_contract_shell(
            context,
            &contract_name,
            solx_mlir::ContractKind::Contract,
            &module_body,
        );

        for state_variable in self.linearised_state_variables() {
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let element_type = AstType::resolve_state_variable(
                &state_variable.get_type().expect("slang validated"),
                context,
            );
            let state = &*context;
            if matches!(slot.location, DataLocation::Immutable) {
                let operation = ImmutableOperation::builder(state.mlir(), state.location())
                    .sym_name(StringAttribute::new(state.mlir(), &slot.name))
                    .r#type(TypeAttribute::new(element_type))
                    .build();
                contract_body.append_operation(operation.into());
                continue;
            }
            let slot_attribute: IntegerAttribute =
                Attribute::parse(state.mlir(), &format!("{} : i256", slot.slot))
                    .expect("valid slot literal")
                    .try_into()
                    .expect("slot literal is an integer attribute");
            let byte_offset_attribute = IntegerAttribute::new(
                IntegerType::new(state.mlir(), solx_utils::BIT_LENGTH_X32 as u32).into(),
                slot.byte_offset.into(),
            );
            let mut operation = StateVarOperation::builder(state.mlir(), state.location())
                .sym_name(StringAttribute::new(state.mlir(), &slot.name))
                .r#type(TypeAttribute::new(element_type))
                .slot(slot_attribute)
                .byte_offset(byte_offset_attribute);
            if matches!(slot.location, DataLocation::Transient) {
                operation = operation.transient(Attribute::unit(state.mlir()));
            }
            contract_body.append_operation(operation.build().into());
        }

        context.current_contract_type = Some(contract_type);
        self.emit_constructor(
            &FunctionScope::new(context, Some(self), &dispatch, &storage_layout),
            &contract_body,
        );
        context.current_contract_type = None;

        let getter_selectors: HashSet<u32> = self
            .linearised_state_variables()
            .iter()
            .filter_map(|state_variable| state_variable.compute_selector())
            .collect();

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
            // An overridden public function colliding with an inherited state-variable auto-getter's
            // selector is skipped here; the getter, emitted last, wins.
            if let Some(selector) = function.compute_selector()
                && getter_selectors.contains(&selector)
            {
                continue;
            }
            context.current_contract_type = Some(contract_type);
            function.emit(
                &FunctionScope::new(context, Some(self), &dispatch, &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }

        for (symbol, function) in &super_dispatch.shadowed {
            context.current_contract_type = Some(contract_type);
            function.emit_with_symbol(
                &FunctionScope::new(context, Some(self), &dispatch, &storage_layout),
                symbol,
                &contract_body,
            );
            context.current_contract_type = None;
        }

        for free in reached_free_functions.iter() {
            context.current_contract_type = Some(contract_type);
            free.emit_with_symbol(
                &FunctionScope::new(context, Some(self), &dispatch, &storage_layout),
                &free.node_id_qualified_symbol(),
                &contract_body,
            );
            context.current_contract_type = None;
        }

        let mut emitted_modifiers: HashSet<NodeId> = HashSet::new();
        let mut invoked_modifiers: Vec<slang_solidity_v2::ast::FunctionDefinition> = Vec::new();
        {
            let modifier_scope =
                FunctionScope::new(context, Some(self), &dispatch, &storage_layout);
            let mut wrapped_functions = self.linearised_functions();
            // Collect modifiers from every constructor in the C3 chain, not just the most-derived
            // contract's own: `emit_constructor` emits a `sol.func` for each base constructor in the
            // linearisation, and each may invoke (override-resolved) modifiers whose `sol.modifier`
            // definitions must be emitted here, or the base constructor's `sol.modifier_call_blk`
            // would reference a dangling symbol (a null MLIR value that crashes lowering).
            for base in self.linearised_bases() {
                if let ContractBase::Contract(base_contract) = base
                    && let Some(constructor) = base_contract.constructor()
                {
                    wrapped_functions.push(constructor);
                }
            }
            for function in wrapped_functions.iter() {
                for modifier in function.resolve_invoked_modifiers(&modifier_scope) {
                    if emitted_modifiers.insert(modifier.node_id()) {
                        invoked_modifiers.push(modifier);
                    }
                }
            }
        }
        for modifier in invoked_modifiers.iter() {
            context.current_contract_type = Some(contract_type);
            modifier.emit_modifier_definition(
                &FunctionScope::new(context, Some(self), &dispatch, &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }

        let environment = Environment::new();
        let getter_context = ExpressionContext::new(
            context,
            &environment,
            &dispatch,
            &storage_layout,
            ArithmeticMode::Checked,
        );
        for state_variable in self.linearised_state_variables() {
            state_variable.emit(&getter_context, contract_body);
        }
    }
}

impl EmitObject for LibraryDefinition {
    /// Emits a deployable library object: its `external` / `public` functions as `sol.func`s.
    ///
    /// A library with no externally-visible function is emitted as an empty, call-protected stub,
    /// kept in the artifacts so the `// library:` directive can deploy and link it.
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

        self.register_signatures(
            context,
            functions
                .iter()
                .map(|function| (function.clone(), function.mlir_function_name())),
        );

        let storage_layout: HashMap<NodeId, StorageSlot> = HashMap::new();
        let dispatch = ContractDispatch::default();
        let library_type = AstType::contract(context.mlir(), &library_name, false).into_mlir();
        let module_body = context.module.body();
        let contract_body = self.emit_contract_shell(
            context,
            &library_name,
            solx_mlir::ContractKind::Library,
            &module_body,
        );

        for function in functions.iter() {
            context.current_contract_type = Some(library_type);
            function.emit(
                &FunctionScope::new(context, None, &dispatch, &storage_layout),
                &contract_body,
            );
            context.current_contract_type = None;
        }
    }
}
