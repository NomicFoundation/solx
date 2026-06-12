//!
//! Function-modifier lowering (modifier-stage `sol.func` chain).
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
use solx_mlir::StateMutability;

use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::body_kind::BodyKind;
use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::function::statement::StatementEmitter;
use crate::ast::type_conversion::TypeConversion;

/// The evaluated arguments of one modifier stage: `(declaration node id, value,
/// type)` per bound modifier parameter — keyed by the parameter's `NodeId` so it
/// binds into the [`NodeId`]-keyed environment. A private support alias (not a
/// top-level type, so §2a is satisfied by the sole [`ModifiedBody`] struct).
pub type ModifierStageParams<'context, 'env> = Vec<(NodeId, Value<'context, 'env>, Type<'context>)>;

/// The frame threaded through the modifier-wrapped emission of one function.
///
/// The SOLE top-level type of this module (§2a) — the references its modifier
/// methods need in common, bundled so `emit_modified_body` takes one frame.
pub struct ModifiedBody<'a, 'context, 'block> {
    /// The function being modifier-wrapped.
    function: &'a FunctionDefinition,
    /// The public entry symbol.
    mlir_name: &'a str,
    /// The entry's MLIR parameter types.
    mlir_parameter_types: &'a [Type<'context>],
    /// The entry's MLIR result types.
    result_types: &'a [Type<'context>],
    /// The `sol.contract` body the stage `sol.func`s are appended to.
    contract_body: &'a BlockRef<'context, 'block>,
    /// The public entry's own entry block.
    function_entry_block: &'a BlockRef<'context, 'block>,
}

impl<'a, 'context, 'block> ModifiedBody<'a, 'context, 'block> {
    /// Bundles the references the modifier emission threads in common.
    pub fn new(
        function: &'a FunctionDefinition,
        mlir_name: &'a str,
        mlir_parameter_types: &'a [Type<'context>],
        result_types: &'a [Type<'context>],
        contract_body: &'a BlockRef<'context, 'block>,
        function_entry_block: &'a BlockRef<'context, 'block>,
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
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
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
        )?;

        // Every return needs a slot so the chain's results can be captured and
        // read back by the epilogue; an unnamed return gets a default-initialised
        // one (a never-reached `_;` then yields the zero default — for a memory
        // aggregate that default must be a fresh zero-filled allocation, not a
        // dangling pointer).
        let return_slang_types: Vec<_> = function
            .returns()
            .map(|returns| {
                returns
                    .iter()
                    .map(|parameter| parameter.get_type())
                    .collect()
            })
            .unwrap_or_default();
        for (index, slot) in return_slots.iter_mut().enumerate() {
            if slot.is_none() {
                let return_type = result_types[index];
                let slang_type = return_slang_types.get(index).and_then(|t| t.as_ref());
                *slot = Some(TypeConversion::emit_default_initialized_slot(
                    slang_type,
                    return_type,
                    &self.state.builder,
                    function_entry_block,
                ));
            }
        }

        // `f`'s own parameters, forwarded unchanged down the chain to the body.
        let function_parameters: Vec<Value<'context, 'block>> = (0..mlir_parameter_types.len())
            .map(|index| function_entry_block.argument(index).map(Into::into))
            .collect::<Result<_, _>>()?;

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
            .map(|params| {
                params
                    .iter()
                    .map(|(_, _, parameter_type)| *parameter_type)
                    .collect()
            })
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
            )?;
        }

        // `f`'s body: call the outermost stage with [all modifier arguments ++
        // f's parameters], threading the current return values (appended by
        // `ModifierBodyCall::emit`) and capturing the results back into the
        // shared return slots, then fall through to `f`'s epilogue.
        let mut forward_params: Vec<Value<'context, 'block>> = Vec::new();
        for params in &modifier_stage_params {
            for (_, pointer, parameter_type) in params {
                forward_params.push(self.state.builder.emit_sol_load(
                    *pointer,
                    *parameter_type,
                    &current_block,
                )?);
            }
        }
        forward_params.extend(function_parameters);
        let body_call = ModifierBodyCall {
            symbol: stage_symbols[0].clone(),
            result_types: result_types.to_vec(),
            forward_params,
            return_slots: return_slots.clone(),
        };
        body_call.emit(&self.state.builder, &current_block)?;
        Ok(Some(current_block))
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
    ) -> anyhow::Result<()> {
        let parameter_types: Vec<Type<'context>> = modifier_params
            .iter()
            .map(|(_, _, parameter_type)| *parameter_type)
            .chain(downstream_types.iter().copied())
            .chain(result_types.iter().copied())
            .collect();

        let entry = self.state.builder.emit_sol_func(
            stage_symbol,
            &parameter_types,
            result_types,
            None,
            StateMutability::NonPayable,
            None,
            None,
            contract_body,
        );
        let region = entry
            .parent_region()
            .expect("entry block belongs to a region");

        // Bind this modifier's parameters from the leading arguments.
        let mut environment = Environment::new();
        for (index, (declaration, _, parameter_type)) in modifier_params.iter().enumerate() {
            let value: Value<'context, '_> = entry.argument(index)?.into();
            let pointer = self.state.builder.emit_sol_alloca(*parameter_type, &entry);
            self.state.builder.emit_sol_store(value, pointer, &entry);
            environment.define_variable(*declaration, pointer, *parameter_type);
        }

        // Downstream values (later modifiers' arguments ++ `f`'s parameters) are
        // forwarded verbatim to the next stage at `_;`.
        let downstream_offset = modifier_params.len();
        let forward_params: Vec<Value<'context, '_>> = (0..downstream_types.len())
            .map(|index| entry.argument(downstream_offset + index).map(Into::into))
            .collect::<Result<_, _>>()?;

        // Return slots, initialised from the threaded-in trailing arguments.
        let return_offset = modifier_params.len() + downstream_types.len();
        let mut return_slots: Vec<Option<Value<'context, '_>>> =
            Vec::with_capacity(result_types.len());
        for (index, &return_type) in result_types.iter().enumerate() {
            let pointer = self.state.builder.emit_sol_alloca(return_type, &entry);
            let incoming: Value<'context, '_> = entry.argument(return_offset + index)?.into();
            self.state.builder.emit_sol_store(incoming, pointer, &entry);
            return_slots.push(Some(pointer));
        }

        let mut emitter = StatementEmitter::new(
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
            match emitter.emit(&statement, current_block)? {
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
        Ok(())
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
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
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
                        let (value, next_block) = {
                            let emitter = ExpressionEmitter::new(
                                self.state,
                                evaluating_scope,
                                self.storage_layout,
                                ArithmeticMode::Checked,
                            );
                            emitter.emit_value(argument, current_block)?
                        };
                        current_block = next_block;
                        let parameter_type = parameter
                            .get_type()
                            .map(|slang_type| {
                                TypeConversion::resolve_slang_type(
                                    &slang_type,
                                    None,
                                    &self.state.builder,
                                )
                            })
                            .unwrap_or_else(|| self.state.builder.types.ui256);
                        let cast = TypeConversion::coerce(
                            value,
                            parameter_type,
                            &self.state.builder,
                            &current_block,
                        );
                        let pointer = self
                            .state
                            .builder
                            .emit_sol_alloca(parameter_type, &current_block);
                        self.state
                            .builder
                            .emit_sol_store(cast, pointer, &current_block);
                        // Bind by the parameter's node id (the recut keys variables
                        // by declaration id, so an unnamed parameter binds harmlessly
                        // and a reference resolves through `resolve_to_definition`).
                        base_environment.define_variable(
                            parameter.node_id(),
                            pointer,
                            parameter_type,
                        );
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
        Ok(current_block)
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
    ) -> anyhow::Result<()> {
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
                self.build_modifier_stages(&base_constructor, environment, current_block)?;
            current_block = next_block;

            if modifier_stages.is_empty() {
                for statement in body.statements().iter() {
                    let mut emitter = StatementEmitter::new(
                        self.state,
                        environment,
                        &region,
                        self.storage_layout,
                        &return_types,
                        &[],
                    );
                    match emitter.emit(&statement, current_block)? {
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
                let mut emitter = StatementEmitter::new(
                    self.state,
                    environment,
                    &region,
                    self.storage_layout,
                    &return_types,
                    &[],
                );
                emitter.set_modifier_stages(modifier_stages, modifier_stage_params);
                match emitter.emit_inline_modifier_chain(current_block)? {
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
            self.state.builder.emit_sol_return(&[], &current_block);
        }
        Ok(())
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
    ///
    /// # Errors
    ///
    /// Returns an error if a modifier-argument expression cannot be lowered.
    pub fn build_modifier_stages<'env>(
        &self,
        function: &FunctionDefinition,
        environment: &Environment<'context, 'env>,
        mut block: BlockRef<'context, 'env>,
    ) -> anyhow::Result<(
        Vec<Statements>,
        Vec<ModifierStageParams<'context, 'env>>,
        BlockRef<'context, 'env>,
    )> {
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
                Self::positional_arguments(invocation.arguments()).unwrap_or_default();
            let mut stage_params: ModifierStageParams<'context, 'env> = Vec::new();
            for (parameter, argument) in modifier_definition
                .parameters()
                .iter()
                .zip(argument_expressions)
            {
                // Evaluate the argument even when the parameter is unnamed
                // (`modifier m(uint) {...}`) — the evaluation may have side
                // effects (`m(f(x))`) that must still run.
                let (value, next_block) = {
                    let emitter = ExpressionEmitter::new(
                        self.state,
                        environment,
                        self.storage_layout,
                        ArithmeticMode::Checked,
                    );
                    emitter.emit_value(&argument, block)?
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
                        TypeConversion::resolve_slang_type(&slang_type, None, &self.state.builder)
                    })
                    .unwrap_or_else(|| self.state.builder.types.ui256);
                let cast =
                    TypeConversion::coerce(value, parameter_type, &self.state.builder, &block);
                let pointer = self.state.builder.emit_sol_alloca(parameter_type, &block);
                self.state.builder.emit_sol_store(cast, pointer, &block);
                stage_params.push((parameter.node_id(), pointer, parameter_type));
            }
            modifier_stages.push(modifier_body.statements());
            modifier_params.push(stage_params);
        }
        Ok((modifier_stages, modifier_params, block))
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
    /// only ever a map key — never string-compared (rule 7).
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
        // Matching by the resolved node id keeps this rule-7-clean — no name
        // comparison — and keys the entry to the linearisation-driven body walk.
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
