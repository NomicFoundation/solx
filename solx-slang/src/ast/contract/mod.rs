//!
//! Contract / library definition emission to Sol dialect MLIR.
//!

pub mod constructor;
pub mod contract_dispatch;
pub mod function;
pub mod getter;
pub mod object_scope;
pub mod storage_layout;

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::ImmutableOperation;
use solx_mlir::ods::sol::StateVarOperation;

use self::contract_dispatch::ContractDispatch;
use self::function::expression::ExpressionContext;
use self::function::expression::arithmetic_mode::ArithmeticMode;
use self::function::mlir_symbol_name::MlirSymbolName;
use self::object_scope::ObjectScope;
use self::storage_layout::StorageSlot;
use crate::ast::analysis::query::storage_layout::StorageLayout;
use crate::ast::analysis::walk::free_function::FreeCallCollector;
use crate::ast::analysis::walk::library::LibraryCallCollector;
use crate::ast::analysis::walk::super_call::SuperDispatch;
use crate::ast::contract::function::function_scope::FunctionScope;
use crate::ast::emit::emit_constructor::EmitConstructor;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_function::EmitFunction;
use crate::ast::emit::emit_modifier_calls::EmitModifierCalls;
use crate::ast::emit::emit_object::EmitObject;

impl EmitObject for ContractDefinition {
    fn emit(&self, context: &mut Context, scope: &ObjectScope) {
        let contract_name = self.name().name();

        let super_dispatch = SuperDispatch::build_super_dispatch(self);
        let dispatch = ContractDispatch::from(&super_dispatch);
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
            AstType::contract(context.mlir_context, &contract_name, self.is_payable()).into_mlir();

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
            if matches!(slot.location, solx_utils::DataLocation::Immutable) {
                mlir_op_void!(
                    state,
                    contract_body,
                    ImmutableOperation
                        .sym_name(StringAttribute::new(state.mlir_context, &slot.name))
                        .r#type(TypeAttribute::new(element_type))
                );
                continue;
            }
            let slot_attribute: IntegerAttribute =
                AstType::signless(state.mlir_context, solx_utils::BIT_LENGTH_FIELD)
                    .big_integer_attribute(&BigInt::from_bytes_be(
                        num_bigint::Sign::Plus,
                        &slot.slot.to_be_bytes_vec(),
                    ))
                    .try_into()
                    .expect("a signless integer attribute");
            let byte_offset_attribute = IntegerAttribute::new(
                IntegerType::new(state.mlir_context, solx_utils::BIT_LENGTH_X32 as u32).into(),
                slot.byte_offset.into(),
            );
            let mut operation = StateVarOperation::builder(state.mlir_context, state.location())
                .sym_name(StringAttribute::new(state.mlir_context, &slot.name))
                .r#type(TypeAttribute::new(element_type))
                .slot(slot_attribute)
                .byte_offset(byte_offset_attribute);
            if matches!(slot.location, solx_utils::DataLocation::Transient) {
                operation = operation.transient(Attribute::unit(state.mlir_context));
            }
            contract_body.append_operation(operation.build().into());
        }

        self.emit_constructor(
            &FunctionScope::new(
                context,
                Some(self),
                Some(contract_type),
                &dispatch,
                &storage_layout,
            ),
            &contract_body,
        );

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
            function.emit(
                &FunctionScope::new(
                    context,
                    Some(self),
                    Some(contract_type),
                    &dispatch,
                    &storage_layout,
                ),
                None,
                &contract_body,
            );
        }

        for (symbol, function) in &super_dispatch.shadowed {
            function.emit(
                &FunctionScope::new(
                    context,
                    Some(self),
                    Some(contract_type),
                    &dispatch,
                    &storage_layout,
                ),
                Some(symbol),
                &contract_body,
            );
        }

        for free in reached_free_functions.iter() {
            free.emit(
                &FunctionScope::new(
                    context,
                    Some(self),
                    Some(contract_type),
                    &dispatch,
                    &storage_layout,
                ),
                Some(free.node_id_qualified_symbol().as_str()),
                &contract_body,
            );
        }

        let mut emitted_modifiers: HashSet<NodeId> = HashSet::new();
        let mut invoked_modifiers: Vec<FunctionDefinition> = Vec::new();
        {
            let modifier_scope = FunctionScope::new(
                context,
                Some(self),
                Some(contract_type),
                &dispatch,
                &storage_layout,
            );
            let mut wrapped_functions = self.linearised_functions();
            // Collect modifiers from every constructor in the C3 chain, not just the most-derived
            // contract's own: `emit_constructor` emits a `sol.func` for each base constructor in the
            // linearisation, and each may invoke override-resolved modifiers whose `sol.modifier`
            // definitions must be emitted here, or the base constructor's `sol.modifier_call_blk`
            // would reference a dangling symbol, a null MLIR value that crashes a later pass.
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
            modifier.emit_modifier_definition(
                &FunctionScope::new(
                    context,
                    Some(self),
                    Some(contract_type),
                    &dispatch,
                    &storage_layout,
                ),
                &contract_body,
            );
        }

        let environment = Environment::new();
        let getter_context = ExpressionContext::new(
            context,
            &environment,
            &dispatch,
            &storage_layout,
            Some(contract_type),
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
        let library_type =
            AstType::contract(context.mlir_context, &library_name, false).into_mlir();
        let module_body = context.module.body();
        let contract_body = self.emit_contract_shell(
            context,
            &library_name,
            solx_mlir::ContractKind::Library,
            &module_body,
        );

        for function in functions.iter() {
            function.emit(
                &FunctionScope::new(
                    context,
                    None,
                    Some(library_type),
                    &dispatch,
                    &storage_layout,
                ),
                None,
                &contract_body,
            );
        }
    }
}
