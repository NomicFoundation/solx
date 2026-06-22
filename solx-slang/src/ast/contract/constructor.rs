//!
//! Contract constructor synthesis: the deploy-time `constructor()` `sol.func`.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitFunction;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::FunctionScope;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::contract::function::statement::modifier_strategy::ModifierStrategy;
use crate::ast::emit::EmitConstructor;
use crate::ast::emit::EmitModifierChain;
use crate::ast::pending_queries::MatchLinearisedBase;
use crate::ast::pending_queries::PositionalArguments;

impl EmitConstructor for ContractDefinition {
    fn emit_constructor<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        contract_body: &BlockRef<'context, '_>,
    ) {
        // C3 linearisation, most-derived (self) first. Interfaces have no
        // constructor, so only contracts contribute to the construction chain.
        let mro: Vec<ContractDefinition> = self
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .collect();

        // With no base constructor, the contract's own constructor (or an empty one running just the
        // state-variable initializers) is the entire construction; otherwise take the chain path below.
        let has_base_constructor = mro.iter().skip(1).any(|base| base.constructor().is_some());
        if !has_base_constructor {
            if let Some(constructor) = self.constructor() {
                constructor.emit(scope, contract_body);
                return;
            }
            let entry = Function::new("constructor()".to_owned(), Vec::new(), Vec::new()).define(
                None,
                StateMutability::NonPayable,
                Some(solx_mlir::FunctionKind::Constructor),
                None,
                &scope.state.builder,
                contract_body,
            );
            let block = self.emit_state_var_initializers(scope, entry);
            mlir_op_void!(&scope.state.builder, &block, ReturnOperation.operands(&[]));
            return;
        }

        // Inheritance chain: one `constructor()` runs every base constructor (base-first), each in its
        // own parameter scope, after the initializers; it takes the most-derived contract's parameters.
        let derived_constructor = self.constructor();
        let (parameter_types, mutability) = match &derived_constructor {
            Some(constructor) => {
                let (parameter_types, _) = AstType::resolve_signature(
                    constructor,
                    LocationPolicy::Declared(None),
                    &scope.state.builder,
                );
                (
                    parameter_types,
                    StateMutability::from(constructor.mutability()),
                )
            }
            None => (Vec::new(), StateMutability::NonPayable),
        };
        let signature = Function::new("constructor()".to_owned(), parameter_types, Vec::new());
        let entry = signature.define(
            None,
            mutability,
            Some(solx_mlir::FunctionKind::Constructor),
            None,
            &scope.state.builder,
            contract_body,
        );

        // Per-contract constructor scopes, keyed by contract node id — base constructors reuse the
        // derived contract's parameter names, so a single flat scope would clobber them.
        let mut root_environment = Environment::new();
        if let Some(constructor) = &derived_constructor {
            for (index, parameter) in constructor.parameters().iter().enumerate() {
                let parameter_type = signature.parameter_types[index];
                let parameter_value = AstValue::new(
                    entry
                        .argument(index)
                        .expect("argument index is within the block signature")
                        .into(),
                );
                let pointer =
                    Pointer::stack_slot(AstType::new(parameter_type), &scope.state.builder, &entry);
                pointer.store(parameter_value, &scope.state.builder, &entry);
                root_environment.define_variable(parameter.node_id(), pointer.into_mlir());
            }
        }

        // State-variable initializers (whole C3 hierarchy) run first; they cannot
        // reference constructor parameters or locals.
        let mut current_block = self.emit_state_var_initializers(scope, entry);

        let mut scopes: HashMap<NodeId, Environment<'context, '_>> = HashMap::new();
        scopes.insert(self.node_id(), root_environment);
        let mut bound_scopes: HashSet<NodeId> = HashSet::new();
        bound_scopes.insert(self.node_id());

        let mro_node_ids: HashSet<NodeId> = mro.iter().map(|base| base.node_id()).collect();
        current_block = self.bind_base_constructor_scopes(
            scope,
            &mro,
            &mro_node_ids,
            &mut scopes,
            &mut bound_scopes,
            current_block,
        );
        self.emit_constructor_bodies(
            scope,
            &mro,
            &mut scopes,
            &bound_scopes,
            &entry,
            current_block,
        )
    }

    fn emit_state_var_initializers<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mut block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        // Initializers cannot reference constructor parameters or locals, so they
        // run over an empty variable environment.
        let environment = Environment::new();
        let emitter = ExpressionContext::new(
            scope.state,
            &environment,
            scope.storage_layout,
            ArithmeticMode::Checked,
        );
        // Run initializers for the whole C3-linearised hierarchy in order, so a derived contract executes
        // its bases' state-variable initializers (including side effects) exactly as solc does.
        for state_variable in self.linearised_state_variables() {
            let Some(slot) = scope.storage_layout.get(&state_variable.node_id()) else {
                continue;
            };
            let Some(initializer) = state_variable.value() else {
                continue;
            };
            let declared_type = state_variable.get_type().expect("slang validated");
            let builder = &scope.state.builder;
            let element_type =
                AstType::resolve(&declared_type, LocationPolicy::Declared(None), builder);
            let address_type =
                AstType::new(element_type).address_type(slot.location, builder.context);
            let storage_ref =
                Pointer::addr_of(&slot.name, address_type, builder, &block).into_mlir();
            let BlockAnd {
                value,
                block: next_block,
            } = initializer.emit(&emitter, block);
            block = next_block;
            if declared_type.is_reference_type() {
                Pointer::new(storage_ref).copy_from(value, builder, &block);
            } else {
                let stored_value = value.cast(AstType::new(element_type), builder, &block);
                Pointer::new(storage_ref).store(stored_value, builder, &block);
            }
        }
        block
    }

    fn bind_base_constructor_scopes<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mro: &[ContractDefinition],
        mro_node_ids: &HashSet<NodeId>,
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &mut HashSet<NodeId>,
        mut current_block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        for contract in mro.iter() {
            // A contract whose constructor takes no parameters evaluates its base-argument expressions
            // in a fresh empty scope; one with parameters was already bound by a more-derived contract.
            scopes.entry(contract.node_id()).or_default();

            let mut base_argument_specs: Vec<(ContractDefinition, Vec<Expression>)> = Vec::new();
            if let Some(constructor) = contract.constructor() {
                for invocation in constructor.modifier_invocations().iter() {
                    let Some(arguments) = invocation
                        .arguments()
                        .and_then(|argument_list| argument_list.positional_arguments())
                    else {
                        continue;
                    };
                    if let Some(base_contract) =
                        invocation.name().match_linearised_base(mro, mro_node_ids)
                    {
                        base_argument_specs.push((base_contract, arguments));
                    }
                }
            }
            for inheritance in contract.inheritance_types().iter() {
                let Some(arguments) = inheritance
                    .arguments()
                    .and_then(|argument_list| argument_list.positional_arguments())
                else {
                    continue;
                };
                if let Some(base_contract) = inheritance
                    .type_name()
                    .match_linearised_base(mro, mro_node_ids)
                {
                    base_argument_specs.push((base_contract, arguments));
                }
            }

            // solc evaluates base-constructor arguments in C3-linearisation order (most-derived first),
            // not source order, so a side-effecting argument runs in the right order.
            base_argument_specs.sort_by_key(|(base, _)| {
                mro.iter()
                    .position(|contract| contract.node_id() == base.node_id())
                    .unwrap_or(usize::MAX)
            });

            // Evaluate the arguments in this contract's scope and build each base's parameter scope.
            // The borrow of the evaluating scope must end before inserting the new scopes, so collect first.
            let evaluated: Vec<(NodeId, Environment<'context, '_>)> = {
                let evaluating_scope = scopes
                    .get(&contract.node_id())
                    .expect("scope ensured at the top of this iteration");
                let mut evaluated = Vec::new();
                for (base_contract, arguments) in base_argument_specs {
                    let base_id = base_contract.node_id();
                    // A more-derived contract already supplied this base's
                    // arguments (most-derived wins).
                    if scopes.contains_key(&base_id) {
                        continue;
                    }
                    let Some(base_constructor) = base_contract.constructor() else {
                        continue;
                    };
                    let mut base_environment = Environment::new();
                    for (parameter, argument) in
                        base_constructor.parameters().iter().zip(arguments.iter())
                    {
                        // Evaluate the argument even for an unnamed parameter — the evaluation may have
                        // side effects that must still run, in base-linearisation order.
                        let BlockAnd {
                            value,
                            block: next_block,
                        } = {
                            let emitter = ExpressionContext::new(
                                scope.state,
                                evaluating_scope,
                                scope.storage_layout,
                                ArithmeticMode::Checked,
                            );
                            argument.emit(&emitter, current_block)
                        };
                        current_block = next_block;
                        let parameter_type = AstType::resolve(
                            &parameter.get_type().expect("slang validated"),
                            LocationPolicy::Declared(None),
                            &scope.state.builder,
                        );
                        let cast = value.cast(
                            AstType::new(parameter_type),
                            &scope.state.builder,
                            &current_block,
                        );
                        let pointer = Pointer::stack_slot(
                            AstType::new(parameter_type),
                            &scope.state.builder,
                            &current_block,
                        );
                        pointer.store(cast, &scope.state.builder, &current_block);
                        // Bind by the parameter's node id (variables are keyed by declaration id, so
                        // an unnamed parameter binds harmlessly).
                        base_environment.define_variable(parameter.node_id(), pointer.into_mlir());
                    }
                    evaluated.push((base_id, base_environment));
                }
                evaluated
            };
            for (base_id, base_environment) in evaluated {
                bound_scopes.insert(base_id);
                scopes.entry(base_id).or_insert(base_environment);
            }
        }
        current_block
    }

    fn emit_constructor_bodies<'state, 'context, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        mro: &[ContractDefinition],
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &HashSet<NodeId>,
        entry: &BlockRef<'context, 'block>,
        mut current_block: BlockRef<'context, 'block>,
    ) {
        let region = entry.parent_region().expect("entry block has a region");
        let return_types: [Type<'context>; 0] = [];
        let mut terminated = false;
        for contract in mro.iter().rev() {
            let Some(base_constructor) = contract.constructor() else {
                continue;
            };
            let Some(body) = base_constructor.body() else {
                continue;
            };
            if !base_constructor.parameters().is_empty()
                && !bound_scopes.contains(&contract.node_id())
            {
                continue;
            }
            let environment = scopes.entry(contract.node_id()).or_default();
            environment.enter_scope();

            // A constructor may carry modifiers, virtually dispatched against the *deployed* contract
            // (an overridden modifier runs its most-derived body). Base invocations resolve to no modifier.
            let (mut modifier_stages, mut modifier_stage_params, next_block) =
                base_constructor.build_modifier_stages(scope, environment, current_block);
            current_block = next_block;

            if modifier_stages.is_empty() {
                for statement in body.statements().iter() {
                    let mut emitter = StatementContext::new(
                        scope.state,
                        environment,
                        &region,
                        scope.storage_layout,
                        &return_types,
                        &[],
                    );
                    match statement.emit(&mut emitter, current_block) {
                        Some(next) => current_block = next,
                        None => {
                            terminated = true;
                            break;
                        }
                    }
                }
            } else {
                // The constructor body is the innermost stage, run inline at the last modifier's `_;`
                // (a constructor has no return value, so the body need not be a separate `sol.func`).
                modifier_stages.push(body.clone());
                modifier_stage_params.push(Vec::new());
                let mut emitter = StatementContext::new(
                    scope.state,
                    environment,
                    &region,
                    scope.storage_layout,
                    &return_types,
                    &[],
                );
                emitter.modifier_strategy = ModifierStrategy::InlineChain {
                    stages: modifier_stages,
                    parameters: modifier_stage_params,
                    index: 0,
                };
                match ModifierStrategy::emit_placeholder(&mut emitter, current_block) {
                    Some(next) => current_block = next,
                    None => terminated = true,
                }
            }

            environment.exit_scope();
            if terminated {
                break;
            }
        }

        if !terminated {
            mlir_op_void!(
                &scope.state.builder,
                &current_block,
                ReturnOperation.operands(&[])
            );
        }
    }
}
