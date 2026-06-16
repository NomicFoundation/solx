//!
//! Function-modifier emission (modifier-stage `sol.func` chain).
//!
//! A modified function `f` is lowered as a chain of internal `sol.func`s —
//! `$mod0 … $modN` (one per modifier invocation, in order) and `$body` (the
//! function's own statements) — each calling the next at its `_` placeholder.
//! The public entry `f` evaluates the modifier arguments and calls `$mod0`.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::IdentifierPath;
use slang_solidity_v2::ast::ModifierInvocation;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statements;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::body_kind::BodyKind;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::function::modifier_parameter_binding::ModifierParameterBinding;
use crate::ast::contract::function::statement::StatementContext;

/// The evaluated arguments of one modifier stage: one
/// [`ModifierParameterBinding`] per bound modifier parameter.
pub type ModifierStageParams<'context, 'env> = Vec<ModifierParameterBinding<'context, 'env>>;

/// The frame threaded through the modifier-wrapped emission of one function —
/// the references its modifier methods need in common, bundled so
/// `emit_modified_body` takes one frame.
pub struct ModifiedBody<'body, 'context, 'block> {
    /// The function being modifier-wrapped.
    function: &'body FunctionDefinition,
    /// The public entry symbol.
    mlir_name: &'body str,
    /// The entry's MLIR parameter types.
    mlir_parameter_types: &'body [Type<'context>],
    /// The entry's MLIR result types.
    result_types: &'body [Type<'context>],
    /// The `sol.contract` body the stage `sol.func`s are appended to.
    contract_body: &'body BlockRef<'context, 'block>,
    /// The public entry's own entry block.
    function_entry_block: &'body BlockRef<'context, 'block>,
}

impl<'body, 'context, 'block> ModifiedBody<'body, 'context, 'block> {
    /// Bundles the references the modifier emission threads in common.
    pub fn new(
        function: &'body FunctionDefinition,
        mlir_name: &'body str,
        mlir_parameter_types: &'body [Type<'context>],
        result_types: &'body [Type<'context>],
        contract_body: &'body BlockRef<'context, 'block>,
        function_entry_block: &'body BlockRef<'context, 'block>,
    ) -> Self {
        Self {
            function,
            mlir_name,
            mlir_parameter_types,
            result_types,
            contract_body,
            function_entry_block,
        }
    }
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Emits a modifier-wrapped function as a chain of internal `sol.func`s.
    ///
    /// Each modifier stage and the wrapped body become their own `sol.func`
    /// (`f$mod0` … `f$modN`, `f$body`), so a `return` inside a modifier emits a
    /// `sol.return` from *that* stage's frame and the parent stage's `_;` site
    /// resumes its tail — matching Solidity (an inlined stage would instead exit
    /// the whole function). `f`'s body then calls `f$mod0` with `[all modifier
    /// arguments ++ f's parameters]` plus the current return values (appended by
    /// [`ModifierBodyCall::emit`]) and captures the results back into the shared
    /// return slots. `environment` is unused: `f`'s parameters are read straight
    /// from the entry block's arguments.
    ///
    /// Returns the block `f` falls through to (always `Some`).
    pub fn emit_modified_body<'frame, 'block>(
        &self,
        frame: &ModifiedBody<'frame, 'context, 'block>,
        environment: &mut Environment<'context, 'block>,
        return_slots: &mut Vec<Option<Value<'context, 'block>>>,
        modifier_stages: Vec<Statements>,
        modifier_stage_params: Vec<ModifierStageParams<'context, 'block>>,
        current_block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        let _ = environment;
        let function = frame.function;
        let mlir_name = frame.mlir_name;
        let mlir_parameter_types = frame.mlir_parameter_types;
        let result_types = frame.result_types;
        let contract_body = frame.contract_body;
        let function_entry_block = frame.function_entry_block;

        // The wrapped body is the innermost func, reached by the last modifier's
        // `_;`. It takes `f`'s parameters plus the threaded-in return values
        // (`BodyKind::ModifierBody`).
        let body_symbol = format!("{mlir_name}$body");
        self.emit_sol_inner(
            function,
            Some(body_symbol.as_str()),
            contract_body,
            BodyKind::ModifierBody,
        );

        // Every return needs a slot so the chain's results can be captured and
        // read back by the epilogue; an unnamed return gets a default-initialised
        // one (a never-reached `_;` then yields the zero default).
        for (index, slot) in return_slots.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(
                    crate::ast::Pointer::default_initialized(
                        crate::ast::Type::new(result_types[index]),
                        &self.state.builder,
                        function_entry_block,
                    )
                    .into_mlir(),
                );
            }
        }

        // `f`'s own parameters, forwarded unchanged down the chain to the body.
        let function_parameters: Vec<Value<'context, 'block>> = (0..mlir_parameter_types.len())
            .map(|index| {
                function_entry_block
                    .argument(index)
                    .expect("argument index is within the block signature")
                    .into()
            })
            .collect();

        // Emit each stage as its own `sol.func`. Stage `i`'s `_;` calls stage
        // `i + 1` (or `$body` for the last). Stage `i`'s "downstream" parameters
        // are every later modifier's argument types followed by `f`'s parameter
        // types — exactly what the next stage binds, so the forward aligns.
        let stage_count = modifier_stages.len();
        let stage_symbols: Vec<String> = (0..stage_count)
            .map(|index| format!("{mlir_name}$mod{index}"))
            .collect();
        let stage_argument_types: Vec<Vec<Type<'context>>> = modifier_stage_params
            .iter()
            .map(|params| params.iter().map(|binding| binding.element_type).collect())
            .collect();
        for index in 0..stage_count {
            let next_symbol = if index + 1 < stage_count {
                stage_symbols[index + 1].as_str()
            } else {
                body_symbol.as_str()
            };
            let downstream_types: Vec<Type<'context>> = stage_argument_types[index + 1..]
                .iter()
                .flatten()
                .copied()
                .chain(mlir_parameter_types.iter().copied())
                .collect();
            self.emit_modifier_stage_func(
                function,
                stage_symbols[index].as_str(),
                &modifier_stages[index],
                &modifier_stage_params[index],
                &downstream_types,
                result_types,
                next_symbol,
                contract_body,
            );
        }

        // `f`'s body: call the outermost stage with [all modifier arguments ++
        // f's parameters], threading the current return values (appended by
        // `ModifierBodyCall::emit`) and capturing the results back into the
        // shared return slots, then fall through to `f`'s epilogue.
        let mut forward_params: Vec<Value<'context, 'block>> = Vec::new();
        for params in modifier_stage_params.iter() {
            for binding in params {
                forward_params.push(
                    crate::ast::Pointer::new(binding.pointer)
                        .load(
                            crate::ast::Type::new(binding.element_type),
                            &self.state.builder,
                            &current_block,
                        )
                        .into_mlir(),
                );
            }
        }
        forward_params.extend(function_parameters);
        let body_call = ModifierBodyCall {
            symbol: stage_symbols[0].clone(),
            result_types: result_types.to_vec(),
            forward_params,
            return_slots: return_slots.clone(),
        };
        body_call.emit(&self.state.builder, &current_block);
        Some(current_block)
    }

    /// Emits one modifier stage as an internal `sol.func`, parameterised by
    /// `[this modifier's arguments ++ downstream_types ++ threaded return
    /// values]`. It binds the modifier's parameters from the leading arguments,
    /// runs the modifier body — whose `_;` calls `next_symbol`, forwarding the
    /// downstream values plus the current return values, and whose `return`
    /// emits a `sol.return` from this frame (resuming the caller's `_;` tail) —
    /// and finishes with the default epilogue when the body falls through.
    pub fn emit_modifier_stage_func(
        &self,
        function: &FunctionDefinition,
        stage_symbol: &str,
        modifier_body: &Statements,
        modifier_params: &ModifierStageParams<'context, '_>,
        downstream_types: &[Type<'context>],
        result_types: &[Type<'context>],
        next_symbol: &str,
        contract_body: &BlockRef<'context, '_>,
    ) {
        let parameter_types: Vec<Type<'context>> = modifier_params
            .iter()
            .map(|binding| binding.element_type)
            .chain(downstream_types.iter().copied())
            .chain(result_types.iter().copied())
            .collect();

        let signature = Function::new(
            stage_symbol.to_owned(),
            parameter_types,
            result_types.to_vec(),
        );
        let entry = signature.define(
            None,
            StateMutability::NonPayable,
            None,
            None,
            &self.state.builder,
            contract_body,
        );
        let region = entry
            .parent_region()
            .expect("entry block belongs to a region");

        // Bind this modifier's parameters from the leading arguments.
        let mut environment = Environment::new();
        for (index, binding) in modifier_params.iter().enumerate() {
            let value = crate::ast::Value::new(
                entry
                    .argument(index)
                    .expect("argument index is within the block signature")
                    .into(),
            );
            let pointer = crate::ast::Pointer::stack_slot(
                crate::ast::Type::new(binding.element_type),
                &self.state.builder,
                &entry,
            );
            pointer.store(value, &self.state.builder, &entry);
            environment.define_variable(binding.declaration, pointer.into_mlir());
        }

        // Downstream values (later modifiers' arguments ++ `f`'s parameters) are
        // forwarded verbatim to the next stage at `_;`.
        let downstream_offset = modifier_params.len();
        let forward_params: Vec<Value<'context, '_>> = (0..downstream_types.len())
            .map(|index| {
                entry
                    .argument(downstream_offset + index)
                    .expect("argument index is within the block signature")
                    .into()
            })
            .collect();

        // Return slots, initialised from the threaded-in trailing arguments.
        let return_offset = modifier_params.len() + downstream_types.len();
        let mut return_slots: Vec<Option<Value<'context, '_>>> =
            Vec::with_capacity(result_types.len());
        for (index, &return_type) in result_types.iter().enumerate() {
            let pointer = crate::ast::Pointer::stack_slot(
                crate::ast::Type::new(return_type),
                &self.state.builder,
                &entry,
            );
            let incoming = crate::ast::Value::new(
                entry
                    .argument(return_offset + index)
                    .expect("argument index is within the block signature")
                    .into(),
            );
            pointer.store(incoming, &self.state.builder, &entry);
            return_slots.push(Some(pointer.into_mlir()));
        }

        let mut emitter = StatementContext::new(
            self.state,
            &mut environment,
            &region,
            self.storage_layout,
            result_types,
            return_slots.as_slice(),
        );
        emitter.set_modifier_body_call(ModifierBodyCall {
            symbol: next_symbol.to_owned(),
            result_types: result_types.to_vec(),
            forward_params,
            return_slots: return_slots.clone(),
        });

        let mut current_block = entry;
        let mut terminated = false;
        for statement in modifier_body.iter() {
            match statement.emit(&mut emitter, current_block) {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }
        if !terminated {
            self.emit_default_return(
                function,
                result_types,
                return_slots.as_slice(),
                &current_block,
            );
        }
    }

    /// Binds each base constructor's parameters into its own scope, in C3 order,
    /// threading the entry block forward (argument evaluation has side effects).
    ///
    /// Each contract's base invocations — its constructor's modifier-style list
    /// (`constructor() Base(args)`) and its inheritance specifiers
    /// (`is Base(args)`) — are matched to their linearised entries and their
    /// argument expressions evaluated in this contract's scope, building each
    /// base's parameter scope. Walking most-derived first means the scope an
    /// argument is evaluated in is already populated (a base's arguments are
    /// written by a more-derived contract and may reference its parameters).
    /// `bound_scopes` records every base whose parameters were bound, so a base
    /// whose arguments could not be matched is skipped during body emission
    /// rather than run against unbound parameters.
    pub fn bind_base_constructor_scopes<'block>(
        &self,
        mro: &[ContractDefinition],
        mro_node_ids: &HashSet<NodeId>,
        scopes: &mut HashMap<NodeId, Environment<'context, 'block>>,
        bound_scopes: &mut HashSet<NodeId>,
        mut current_block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        for contract in mro.iter() {
            // A contract whose constructor takes no externally-supplied
            // parameters evaluates its own base-argument expressions in a fresh
            // empty scope; one with parameters was already bound by a more-derived
            // contract (this leaves that scope untouched).
            scopes.entry(contract.node_id()).or_default();

            let mut base_argument_specs: Vec<(ContractDefinition, Vec<Expression>)> = Vec::new();
            if let Some(constructor) = contract.constructor() {
                for invocation in constructor.modifier_invocations().iter() {
                    let Some(arguments) = Self::positional_arguments(invocation.arguments()) else {
                        continue;
                    };
                    if let Some(base_contract) =
                        Self::match_linearised_base(&invocation.name(), mro, mro_node_ids)
                    {
                        base_argument_specs.push((base_contract, arguments));
                    }
                }
            }
            for inheritance in contract.inheritance_types().iter() {
                let Some(arguments) = Self::positional_arguments(inheritance.arguments()) else {
                    continue;
                };
                if let Some(base_contract) =
                    Self::match_linearised_base(&inheritance.type_name(), mro, mro_node_ids)
                {
                    base_argument_specs.push((base_contract, arguments));
                }
            }

            // solc evaluates base-constructor arguments in C3-linearisation order
            // (most-derived base first), not source order, so a side-effecting
            // argument runs in the right order. Pure arguments are
            // order-insensitive, so this is invisible to value-only base-ctor
            // tests.
            base_argument_specs.sort_by_key(|(base, _)| {
                mro.iter()
                    .position(|contract| contract.node_id() == base.node_id())
                    .unwrap_or(usize::MAX)
            });

            // Evaluate the arguments in this contract's scope and build each
            // base's parameter scope. The immutable borrow of the evaluating
            // scope must end before the new scopes are inserted, so collect first.
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
                        // Evaluate the argument even when the parameter is unnamed
                        // (`constructor(uint)`) — the evaluation may have side
                        // effects that must still run, in base-linearisation order.
                        let BlockAnd {
                            value,
                            block: next_block,
                        } = {
                            let emitter = ExpressionContext::new(
                                self.state,
                                evaluating_scope,
                                self.storage_layout,
                                ArithmeticMode::Checked,
                            );
                            argument.emit(&emitter, current_block)
                        };
                        current_block = next_block;
                        let parameter_type = parameter
                            .get_type()
                            .map(|slang_type| {
                                crate::ast::Type::resolve(
                                    &slang_type,
                                    LocationPolicy::Declared(None),
                                    &self.state.builder,
                                )
                            })
                            .unwrap_or_else(|| {
                                crate::ast::Type::unsigned(
                                    self.state.builder.context,
                                    solx_utils::BIT_LENGTH_FIELD,
                                )
                                .into_mlir()
                            });
                        let cast = value.coerce_to(
                            crate::ast::Type::new(parameter_type),
                            &self.state.builder,
                            &current_block,
                        );
                        let pointer = crate::ast::Pointer::stack_slot(
                            crate::ast::Type::new(parameter_type),
                            &self.state.builder,
                            &current_block,
                        );
                        pointer.store(cast, &self.state.builder, &current_block);
                        // Bind by the parameter's node id (the recut keys variables
                        // by declaration id, so an unnamed parameter binds harmlessly
                        // and a reference resolves through `resolve_to_definition`).
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

    /// Emits each base constructor's body, base-first (reversed MRO), each in its
    /// own parameter scope, then finishes the constructor with a `sol.return`
    /// unless a body already terminated the block. A base whose constructor takes
    /// parameters that were never bound (its arguments could not be matched) is
    /// skipped — its body would reference unbound parameters.
    pub fn emit_constructor_bodies<'block>(
        &self,
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

            // A constructor may carry modifiers (`constructor() mod1`). They are
            // virtually dispatched against the *deployed* contract (resolved by
            // `build_modifier_stages`, so an overridden modifier runs its
            // most-derived body even while a base constructor executes). Base
            // invocations in the same list (`Base(args)`) resolve to no modifier
            // and are skipped — their arguments were already bound above.
            let (mut modifier_stages, mut modifier_stage_params, next_block) =
                self.build_modifier_stages(&base_constructor, environment, current_block);
            current_block = next_block;

            if modifier_stages.is_empty() {
                for statement in body.statements().iter() {
                    let mut emitter = StatementContext::new(
                        self.state,
                        environment,
                        &region,
                        self.storage_layout,
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
                // The constructor body is the innermost stage: the last modifier's
                // `_;` runs it inline. A constructor has no return value, so the
                // body need not be a separate `sol.func`.
                modifier_stages.push(body.statements());
                modifier_stage_params.push(Vec::new());
                let mut emitter = StatementContext::new(
                    self.state,
                    environment,
                    &region,
                    self.storage_layout,
                    &return_types,
                    &[],
                );
                emitter.set_modifier_stages(modifier_stages, modifier_stage_params);
                match emitter.emit_inline_modifier_chain(current_block) {
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
            sol_op_void!(
                &self.state.builder,
                &current_block,
                ReturnOperation.operands(&[])
            );
        }
    }

    /// Resolves the modifier invocations on `function` to their modifier bodies,
    /// evaluating each invocation's arguments in `environment` (the clean
    /// function scope) and storing them into fresh per-invocation allocas.
    ///
    /// Returns, in source order (outermost modifier first): the modifier-body
    /// statement stages, the parallel per-stage parameter bindings (`(node id,
    /// pointer, type)`, bound in a scope local to each stage), and the block
    /// after the argument evaluations. Arguments are evaluated against
    /// `environment` *without* registering any modifier parameter, so an
    /// argument referencing a name resolves to the function's variable and a
    /// repeated modifier keeps a distinct binding per use. Plain (non-virtual,
    /// non-qualified) invocations resolve directly; virtual override re-dispatch
    /// and namespace-qualified paths are inheritance-only (the C3 cluster), and
    /// an invocation that does not resolve to a modifier (a base-constructor call
    /// `Base(args)`) is skipped here and emitted by the constructor path.
    pub fn build_modifier_stages<'env>(
        &self,
        function: &FunctionDefinition,
        environment: &Environment<'context, 'env>,
        mut block: BlockRef<'context, 'env>,
    ) -> (
        Vec<Statements>,
        Vec<ModifierStageParams<'context, 'env>>,
        BlockRef<'context, 'env>,
    ) {
        let mut modifier_stages: Vec<Statements> = Vec::new();
        let mut modifier_params: Vec<ModifierStageParams<'context, 'env>> = Vec::new();
        for invocation in function.modifier_invocations().iter() {
            // Resolve the invocation lexically (a plain `m`), else by its final
            // path segment (a namespace-qualified `M.C.m`); a base-constructor
            // invocation `Base(args)` resolves to neither and is skipped here.
            let resolved_modifier = match invocation.name().resolve_to_definition() {
                Some(Definition::Modifier(modifier)) => modifier,
                _ => match self.resolve_qualified_modifier(&invocation) {
                    Some(modifier) => modifier,
                    None => continue,
                },
            };
            // Re-dispatch to the most-derived override of this modifier (a
            // `virtual` modifier overridden in a derived contract); a non-virtual
            // or library modifier keeps its lexical resolution.
            let modifier_definition = self
                .resolve_modifier_override(&invocation, &resolved_modifier)
                .unwrap_or(resolved_modifier);
            let Some(modifier_body) = modifier_definition.body() else {
                continue;
            };
            let argument_expressions =
                Self::positional_arguments(invocation.arguments()).unwrap_or_default(); // recut-lint-allow: fail01 — a modifier invocation may carry no arguments
            let mut stage_params: ModifierStageParams<'context, 'env> = Vec::new();
            for (parameter, argument) in modifier_definition
                .parameters()
                .iter()
                .zip(argument_expressions)
            {
                // Evaluate the argument even when the parameter is unnamed
                // (`modifier m(uint) {...}`) — the evaluation may have side
                // effects (`m(f(x))`) that must still run.
                let BlockAnd {
                    value,
                    block: next_block,
                } = {
                    let emitter = ExpressionContext::new(
                        self.state,
                        environment,
                        self.storage_layout,
                        ArithmeticMode::Checked,
                    );
                    argument.emit(&emitter, block)
                };
                block = next_block;
                // An unnamed modifier parameter is never referenced in the body,
                // so it gets no binding (the argument was still evaluated above
                // for its side effects).
                if parameter.name().is_none() {
                    continue;
                }
                let parameter_type = parameter
                    .get_type()
                    .map(|slang_type| {
                        crate::ast::Type::resolve(
                            &slang_type,
                            LocationPolicy::Declared(None),
                            &self.state.builder,
                        )
                    })
                    .unwrap_or_else(|| {
                        crate::ast::Type::unsigned(
                            self.state.builder.context,
                            solx_utils::BIT_LENGTH_FIELD,
                        )
                        .into_mlir()
                    });
                let cast = value.coerce_to(
                    crate::ast::Type::new(parameter_type),
                    &self.state.builder,
                    &block,
                );
                let pointer = crate::ast::Pointer::stack_slot(
                    crate::ast::Type::new(parameter_type),
                    &self.state.builder,
                    &block,
                );
                pointer.store(cast, &self.state.builder, &block);
                stage_params.push(ModifierParameterBinding {
                    declaration: parameter.node_id(),
                    pointer: pointer.into_mlir(),
                    element_type: parameter_type,
                });
            }
            modifier_stages.push(modifier_body.statements());
            modifier_params.push(stage_params);
        }
        (modifier_stages, modifier_params, block)
    }

    /// Re-dispatches a virtual modifier invocation to its most-derived
    /// implementation with a body (qualified invocations resolve directly).
    ///
    /// A `virtual` modifier may be declared abstract (or with a base body) in a
    /// base and `override`-n in a derived contract; a lexical invocation picks
    /// the base declaration, so re-resolve against the contract's C3-linearised
    /// modifiers (most-derived first). Returns `None` — keep the lexical
    /// resolution — when the invocation is qualified (`Base.m`, which names a
    /// specific modifier and bypasses virtual dispatch) or when the resolved
    /// modifier is not part of this contract's hierarchy (e.g. a library
    /// modifier reached through `using L for *`, which must not be
    /// virtual-dispatched against a same-named modifier of the using contract).
    pub fn resolve_modifier_override(
        &self,
        invocation: &ModifierInvocation,
        resolved: &FunctionDefinition,
    ) -> Option<FunctionDefinition> {
        if invocation.name().len() > 1 {
            return None;
        }
        let resolved_id = resolved.node_id();
        if !self
            .linearised_modifiers()
            .iter()
            .any(|modifier| modifier.node_id() == resolved_id)
        {
            return None;
        }
        let name = resolved.name().map(|identifier| identifier.name())?;
        self.most_derived_modifiers_by_name().get(&name).cloned()
    }

    /// Resolves a qualified modifier invocation by last-segment name against the
    /// C3 modifiers; `None` marks a base-constructor invocation.
    ///
    /// A namespace-qualified path (`M.M.C.m`) does not resolve to a definition
    /// directly, so its final segment (the modifier name) is matched against the
    /// contract's C3-linearised modifiers, preferring the most-derived one with a
    /// body. `None` when no modifier of that name exists — in particular a
    /// base-constructor invocation `Base(args)`, whose final segment is a
    /// contract name, so the caller leaves it to the constructor path.
    pub fn resolve_qualified_modifier(
        &self,
        invocation: &ModifierInvocation,
    ) -> Option<FunctionDefinition> {
        let modifier_name = invocation.name().iter().last()?.name();
        self.most_derived_modifiers_by_name()
            .get(&modifier_name)
            .cloned()
    }

    /// Every modifier across the contract's C3-linearised bases (most-derived
    /// first). Empty in a library context (no contract / no inheritance).
    fn linearised_modifiers(&self) -> Vec<FunctionDefinition> {
        let Some(contract) = self.contract else {
            return Vec::new();
        };
        contract
            .linearised_bases()
            .into_iter()
            .filter_map(|base| match base {
                ContractBase::Contract(base_contract) => Some(base_contract),
                ContractBase::Interface(_) => None,
            })
            .flat_map(|base_contract| base_contract.modifiers())
            .collect()
    }

    /// The most-derived modifier with a body, per name, across the contract's C3
    /// linearisation. Modifiers cannot be overloaded, so the name uniquely keys
    /// an override chain; `linearised_bases` is most-derived first, so the first
    /// body-bearing modifier of each name is the active override. The name is
    /// only ever a map key — never string-compared.
    fn most_derived_modifiers_by_name(&self) -> HashMap<String, FunctionDefinition> {
        let mut by_name: HashMap<String, FunctionDefinition> = HashMap::new();
        for modifier in self.linearised_modifiers() {
            if modifier.body().is_none() {
                continue;
            }
            let Some(name) = modifier.name().map(|identifier| identifier.name()) else {
                continue;
            };
            by_name.entry(name).or_insert(modifier);
        }
        by_name
    }

    /// Resolves an `IdentifierPath` modifier/base reference to a contract in the
    /// MRO (by definition, else by the aliased last-segment name).
    ///
    /// The path in `is Base` or in a base-constructor invocation `Base(args)`
    /// resolves to a contract definition; if that contract is in the
    /// linearisation, its `mro` entry is returned so callers key scopes
    /// consistently with the linearisation-driven body walk. An import-aliased
    /// path (`M.C`) does not resolve to a definition on its own, so it falls
    /// back to matching the final path segment's name against the linearised
    /// contracts (the alias renames the namespace, not the contract).
    pub fn match_linearised_base(
        path: &IdentifierPath,
        mro: &[ContractDefinition],
        mro_node_ids: &HashSet<NodeId>,
    ) -> Option<ContractDefinition> {
        // Resolve the path to its contract definition: the whole path
        // (`Base`), else its final segment (`M.Base` — an import-aliased path
        // does not resolve as a whole, but its last segment names the contract).
        // Matching by the resolved node id needs no name comparison and keys the
        // entry to the linearisation-driven body walk.
        let base_definition = path
            .resolve_to_definition()
            .or_else(|| path.iter().last()?.resolve_to_definition());
        let Some(Definition::Contract(base_contract)) = base_definition else {
            return None;
        };
        if !mro_node_ids.contains(&base_contract.node_id()) {
            return None;
        }
        mro.iter()
            .find(|contract| contract.node_id() == base_contract.node_id())
            .cloned()
    }

    /// Extracts the positional arguments of a modifier/base invocation, or `None`
    /// when the argument list is empty / absent. Shared by the modifier-stage
    /// resolution and (later) the base-constructor path.
    pub fn positional_arguments(
        arguments: Option<ArgumentsDeclaration>,
    ) -> Option<Vec<Expression>> {
        match arguments {
            Some(ArgumentsDeclaration::PositionalArguments(positional)) => {
                let expressions: Vec<Expression> = positional.iter().collect();
                (!expressions.is_empty()).then_some(expressions)
            }
            _ => None,
        }
    }
}
