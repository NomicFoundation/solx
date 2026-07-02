//!
//! Contract definition lowering to Sol dialect MLIR.
//!

/// Contract-local super/base and virtual dispatch maps.
pub mod contract_dispatch;
/// Function definition lowering to Sol dialect MLIR.
pub mod function;
/// Public state-variable getter synthesis.
pub mod getter;

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
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::FunctionVisibility;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::ContractOperation;
use solx_mlir::ods::sol::ImmutableOperation;
use solx_mlir::ods::sol::StateVarOperation;

use crate::ast::analysis::query::storage_layout::StorageLayout;
use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::analysis::walk::free_function::FreeCallCollector;
use crate::ast::analysis::walk::library::LibraryCallCollector;
use crate::ast::analysis::walk::super_call::SuperDispatch;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::emit::emit_constructor::EmitConstructor;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_function::EmitFunction;
use crate::ast::emit::emit_modifier_calls::EmitModifierCalls;
use crate::ast::emit::emit_object::EmitLibrary;
use crate::ast::emit::emit_object::EmitObject;

use self::function::FunctionEmitter;
use self::function::expression::ExpressionContext;
use self::function::expression::call::type_conversion::TypeConversion;

/// Lowers a Solidity contract to Sol dialect MLIR.
///
/// Emits `sol.contract` wrapping `sol.func` definitions. The
/// `convert-sol-to-yul` pass generates the entry-point dispatcher
/// from the function selectors.
pub struct ContractEmitter;

impl ContractEmitter {
    /// Returns whether `contract` is payable (declares a `receive()` function or
    /// a `payable` `fallback()` function). Single source of truth for payability
    /// derivation — used both when emitting the `sol.contract` op and when
    /// resolving `SlangType::Contract` to a `Sol_ContractType`.
    pub fn is_contract_payable(contract: &ContractDefinition) -> bool {
        contract.functions().iter().any(|function| {
            matches!(function.kind(), FunctionKind::Receive)
                || (matches!(function.kind(), FunctionKind::Fallback)
                    && matches!(function.mutability(), FunctionMutability::Payable))
        })
    }

    /// The functions a contract emits as its own `sol.func`s, call-resolution order preserved: its own
    /// declarations in source order, followed by the base-contract functions it inherits unchanged in
    /// C3-linearisation order. An inherited function overridden in a more-derived contract is excluded,
    /// its most-derived override already present among the own declarations or a nearer base.
    fn emitted_functions(contract: &ContractDefinition) -> Vec<FunctionDefinition> {
        let mut functions = contract.functions();
        let own_ids: HashSet<NodeId> = functions
            .iter()
            .map(|function| function.node_id())
            .collect();
        for function in contract.linearised_functions() {
            if matches!(function.kind(), FunctionKind::Modifier) {
                continue;
            }
            if own_ids.contains(&function.node_id()) {
                continue;
            }
            functions.push(function);
        }
        functions
    }

    /// Pre-registers every function signature reachable for call resolution before bodies are
    /// emitted, so a call to an inherited or virtually dispatched method resolves to a registered
    /// signature.
    fn pre_register_functions(context: &mut Context, contract: &ContractDefinition) {
        for function in Self::emitted_functions(contract) {
            if matches!(function.kind(), FunctionKind::Modifier) {
                continue;
            }
            Self::register_function(context, &function);
        }
    }

    /// Registers a shadowed base override under its contract-qualified symbol, so a `super.f()` /
    /// `Base.f()` call resolving to it reaches the `sol.func` emitted under the same symbol.
    fn register_shadowed_function(
        context: &mut Context,
        function: &FunctionDefinition,
        symbol: &str,
    ) {
        let (parameter_types, return_types) =
            TypeConversion::resolve_function_types(function, context);
        context.register_function_signature(
            function.node_id(),
            symbol.to_owned(),
            parameter_types,
            return_types,
        );
    }

    /// Registers one function's MLIR signature keyed by its AST definition, so a call resolves before
    /// any body is emitted.
    fn register_function(context: &mut Context, function: &FunctionDefinition) {
        let mlir_name = FunctionEmitter::mlir_function_name(function);
        let (parameter_types, return_types) =
            TypeConversion::resolve_function_types(function, context);
        context.register_function_signature(
            function.node_id(),
            mlir_name,
            parameter_types,
            return_types,
        );
    }

    /// Registers a reached free function under its node-id-qualified symbol, so a bare-identifier
    /// reference or operator dispatch to it resolves to the same symbol its `sol.func` carries.
    fn register_free_function(context: &mut Context, function: &FunctionDefinition) {
        let mlir_name = FunctionEmitter::free_function_symbol(function);
        let (parameter_types, return_types) =
            TypeConversion::resolve_function_types(function, context);
        context.register_function_signature(
            function.node_id(),
            mlir_name,
            parameter_types,
            return_types,
        );
    }
}

impl EmitObject for ContractDefinition {
    /// Emits a `sol.contract` containing all function definitions, followed by the reachable free
    /// functions the contract references.
    fn emit(
        &self,
        context: &mut Context,
        operator_functions: &[FunctionDefinition],
        free_functions: &[FunctionDefinition],
    ) {
        let contract_name = self.name().name();

        let super_dispatch = SuperDispatch::build_super_dispatch(self);
        let dispatch = ContractDispatch::from(&super_dispatch);
        let shadowed_functions: Vec<FunctionDefinition> = super_dispatch
            .shadowed
            .iter()
            .map(|(_, function)| function.clone())
            .collect();

        let mut walk_roots = operator_functions.to_vec();
        walk_roots.extend(shadowed_functions.iter().cloned());
        let library_functions =
            LibraryCallCollector::reachable_library_functions(self, free_functions, &walk_roots);
        walk_roots.extend(library_functions.iter().cloned());

        let mut reached_free_functions =
            FreeCallCollector::reachable_free_functions(self, free_functions, &walk_roots);
        let mut seen: HashSet<NodeId> = reached_free_functions
            .iter()
            .map(|function| function.node_id())
            .collect();
        for function in operator_functions.iter().cloned().chain(library_functions) {
            if seen.insert(function.node_id()) {
                reached_free_functions.push(function);
            }
        }

        ContractEmitter::pre_register_functions(context, self);
        for (symbol, function) in super_dispatch.shadowed.iter() {
            ContractEmitter::register_shadowed_function(context, function, symbol);
        }
        for function in &reached_free_functions {
            ContractEmitter::register_free_function(context, function);
        }
        let storage_layout = self.storage_layout();

        let contract_type = AstType::contract(
            context.mlir_context,
            &contract_name,
            ContractEmitter::is_contract_payable(self),
        )
        .into_mlir();

        let module_body = context.module.body();
        let contract_body = mlir_region_op!(
            context, &module_body,
            ContractOperation
                .sym_name(StringAttribute::new(context.mlir_context, &contract_name))
                .kind(solx_mlir::ContractKind::Contract.attribute(context.mlir_context));
            body_region
        );

        for state_variable in self.linearised_state_variables() {
            let Some(slot) = storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let element_type =
                TypeConversion::resolve_state_variable_type(&state_variable, context);
            if matches!(slot.location, solx_utils::DataLocation::Immutable) {
                contract_body.append_operation(
                    ImmutableOperation::builder(context.mlir_context, context.location())
                        .sym_name(StringAttribute::new(context.mlir_context, &slot.name))
                        .r#type(TypeAttribute::new(element_type))
                        .build()
                        .into(),
                );
                continue;
            }
            let slot_attribute: IntegerAttribute =
                Attribute::parse(context.mlir_context, &format!("{} : i256", slot.slot))
                    .expect("valid slot literal")
                    .try_into()
                    .expect("slot literal is an integer attribute");
            let byte_offset_attribute = IntegerAttribute::new(
                IntegerType::new(context.mlir_context, solx_utils::BIT_LENGTH_X32 as u32).into(),
                slot.byte_offset.into(),
            );
            let mut operation = StateVarOperation::builder(context.mlir_context, context.location())
                .sym_name(StringAttribute::new(context.mlir_context, &slot.name))
                .r#type(TypeAttribute::new(element_type))
                .slot(slot_attribute)
                .byte_offset(byte_offset_attribute);
            if matches!(slot.location, solx_utils::DataLocation::Transient) {
                operation = operation.transient(Attribute::unit(context.mlir_context));
            }
            contract_body.append_operation(operation.build().into());
        }

        context.current_contract_type = Some(contract_type);
        self.emit_constructor(
            &FunctionEmitter::new(context, Some(self), &storage_layout, &dispatch),
            &contract_body,
        );
        context.current_contract_type = None;

        let getter_selectors: HashSet<u32> = self
            .linearised_state_variables()
            .iter()
            .filter_map(|state_variable| state_variable.compute_selector())
            .collect();

        for function in ContractEmitter::emitted_functions(self) {
            if let Some(selector) = function.compute_selector()
                && getter_selectors.contains(&selector)
            {
                continue;
            }
            context.current_contract_type = Some(contract_type);
            function.emit(
                &FunctionEmitter::new(context, Some(self), &storage_layout, &dispatch),
                None,
                &contract_body,
            );
            context.current_contract_type = None;
        }

        for (symbol, function) in super_dispatch.shadowed.iter() {
            context.current_contract_type = Some(contract_type);
            function.emit(
                &FunctionEmitter::new(context, Some(self), &storage_layout, &dispatch),
                Some(symbol.as_str()),
                &contract_body,
            );
            context.current_contract_type = None;
        }

        for function in &reached_free_functions {
            let symbol = FunctionEmitter::free_function_symbol(function);
            function.emit(
                &FunctionEmitter::new(context, Some(self), &storage_layout, &dispatch),
                Some(symbol.as_str()),
                &contract_body,
            );
        }

        let mut emitted_modifiers: HashSet<NodeId> = HashSet::new();
        let mut invoked_modifiers: Vec<FunctionDefinition> = Vec::new();
        {
            let modifier_emitter =
                FunctionEmitter::new(context, Some(self), &storage_layout, &dispatch);
            let mut wrapped_functions = self.linearised_functions();
            for base in self.linearised_bases() {
                if let ContractBase::Contract(base_contract) = base
                    && let Some(constructor) = base_contract.constructor()
                {
                    wrapped_functions.push(constructor);
                }
            }
            wrapped_functions.extend(reached_free_functions.iter().cloned());
            for function in wrapped_functions.iter() {
                for modifier in function.resolve_invoked_modifiers(&modifier_emitter) {
                    if emitted_modifiers.insert(modifier.node_id()) {
                        invoked_modifiers.push(modifier);
                    }
                }
            }
        }
        context.current_contract_type = Some(contract_type);
        for modifier in invoked_modifiers.iter() {
            modifier.emit_modifier_definition(
                &FunctionEmitter::new(context, Some(self), &storage_layout, &dispatch),
                &contract_body,
            );
        }
        context.current_contract_type = None;

        context.current_contract_type = Some(contract_type);
        {
            let environment = Environment::new();
            let getter_context =
                ExpressionContext::new(context, &environment, &storage_layout, &dispatch, true);
            for state_variable in self.linearised_state_variables() {
                state_variable.emit(&getter_context, contract_body);
            }
        }
        context.current_contract_type = None;
    }
}

impl EmitLibrary for LibraryDefinition {
    /// Emits a deployable library object: its `external` / `public` functions as `sol.func`s inside a
    /// `sol.contract` of library kind, so the `// library:` directive can deploy and link it.
    fn emit(&self, context: &mut Context) {
        let library_name = self.name().name();

        let functions: Vec<FunctionDefinition> = self
            .members()
            .iter()
            .filter_map(|member| match member {
                ContractMember::FunctionDefinition(function)
                    if matches!(function.kind(), FunctionKind::Regular)
                        && matches!(
                            function.visibility(),
                            FunctionVisibility::External | FunctionVisibility::Public
                        ) =>
                {
                    Some(function.clone())
                }
                _ => None,
            })
            .collect();

        for function in &functions {
            ContractEmitter::register_function(context, function);
        }

        let storage_layout: HashMap<NodeId, StorageSlot> = HashMap::new();
        let dispatch = ContractDispatch::default();
        let library_type =
            AstType::contract(context.mlir_context, &library_name, false).into_mlir();

        let module_body = context.module.body();
        let contract_body = mlir_region_op!(
            context, &module_body,
            ContractOperation
                .sym_name(StringAttribute::new(context.mlir_context, &library_name))
                .kind(solx_mlir::ContractKind::Library.attribute(context.mlir_context));
            body_region
        );

        context.current_contract_type = Some(library_type);
        for function in &functions {
            function.emit(
                &FunctionEmitter::new(context, None, &storage_layout, &dispatch),
                None,
                &contract_body,
            );
        }
        context.current_contract_type = None;
    }
}
