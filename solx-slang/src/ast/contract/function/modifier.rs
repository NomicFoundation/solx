//!
//! Function-modifier emission (modifier-stage `sol.func` chain).
//!
//! A modified function `f` is lowered as a chain of internal `sol.func`s —
//! `$mod0 … $modN` (one per modifier invocation, in order) and `$body` (the
//! function's own statements) — each calling the next at its `_` placeholder.
//! The public entry `f` evaluates the modifier arguments and calls `$mod0`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::Block;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Environment;
use solx_mlir::Function;
use solx_mlir::StateMutability;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitFunction;
use crate::ast::EmitStatement;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::FunctionScope;
use crate::ast::contract::function::body_kind::BodyKind;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::function::modifier_parameter_binding::ModifierParameterBinding;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::contract::function::statement::modifier_strategy::ModifierStrategy;
use crate::ast::emit::EmitModifierChain;
use crate::ast::pending_queries::ModifierResolution;
use crate::ast::pending_queries::PositionalArguments;

/// The evaluated arguments of one modifier stage: one
/// [`ModifierParameterBinding`] per bound modifier parameter.
pub type ModifierStageParams<'context, 'env> = Vec<ModifierParameterBinding<'context, 'env>>;

/// The frame threaded through the modifier-wrapped emission of one function —
/// the references the modifier-chain emission needs in common, bundled so
/// `emit_modified_body` takes one frame. The wrapped function itself is the
/// `EmitModifierChain` receiver, so it is not carried here.
pub struct ModifiedBody<'body, 'context, 'block> {
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
        mlir_name: &'body str,
        mlir_parameter_types: &'body [Type<'context>],
        result_types: &'body [Type<'context>],
        contract_body: &'body BlockRef<'context, 'block>,
        function_entry_block: &'body BlockRef<'context, 'block>,
    ) -> Self {
        Self {
            mlir_name,
            mlir_parameter_types,
            result_types,
            contract_body,
            function_entry_block,
        }
    }
}

impl EmitModifierChain for FunctionDefinition {
    fn emit_modified_body<'state, 'context, 'frame, 'block>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        frame: &ModifiedBody<'frame, 'context, 'block>,
        environment: &mut Environment<'context, 'block>,
        return_slots: &mut Vec<Option<Value<'context, 'block>>>,
        modifier_stages: Vec<Block>,
        modifier_stage_params: Vec<ModifierStageParams<'context, 'block>>,
        current_block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        // Unused: `f`'s parameters are read straight from the entry block's
        // arguments, not from the variable environment.
        let _ = environment;
        let mlir_name = frame.mlir_name;
        let mlir_parameter_types = frame.mlir_parameter_types;
        let result_types = frame.result_types;
        let contract_body = frame.contract_body;
        let function_entry_block = frame.function_entry_block;

        // The wrapped body is the innermost func, reached by the last modifier's
        // `_;`. It takes `f`'s parameters plus the threaded-in return values
        // (`BodyKind::ModifierBody`).
        let body_symbol = format!("{mlir_name}$body");
        self.emit_inner(
            scope,
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
                    Pointer::default_initialized(
                        AstType::new(result_types[index]),
                        &scope.state.builder,
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
                scope,
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
                    Pointer::new(binding.pointer)
                        .load(
                            AstType::new(binding.element_type),
                            &scope.state.builder,
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
        body_call.emit(&scope.state.builder, &current_block);
        Some(current_block)
    }

    fn emit_modifier_stage_func<'state, 'context>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        stage_symbol: &str,
        modifier_body: &Block,
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
            &scope.state.builder,
            contract_body,
        );
        let region = entry
            .parent_region()
            .expect("entry block belongs to a region");

        // Bind this modifier's parameters from the leading arguments.
        let mut environment = Environment::new();
        for (index, binding) in modifier_params.iter().enumerate() {
            let value = AstValue::new(
                entry
                    .argument(index)
                    .expect("argument index is within the block signature")
                    .into(),
            );
            let pointer = Pointer::stack_slot(
                AstType::new(binding.element_type),
                &scope.state.builder,
                &entry,
            );
            pointer.store(value, &scope.state.builder, &entry);
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
            let pointer =
                Pointer::stack_slot(AstType::new(return_type), &scope.state.builder, &entry);
            let incoming = AstValue::new(
                entry
                    .argument(return_offset + index)
                    .expect("argument index is within the block signature")
                    .into(),
            );
            pointer.store(incoming, &scope.state.builder, &entry);
            return_slots.push(Some(pointer.into_mlir()));
        }

        let mut emitter = StatementContext::new(
            scope.state,
            &mut environment,
            &region,
            scope.storage_layout,
            result_types,
            return_slots.as_slice(),
        );
        emitter.modifier_strategy = ModifierStrategy::BodyCall(ModifierBodyCall {
            symbol: next_symbol.to_owned(),
            result_types: result_types.to_vec(),
            forward_params,
            return_slots: return_slots.clone(),
        });

        // A regular-function modifier stage iterates the body statements
        // directly rather than through `Block::emit`: the stage func owns the
        // `terminated` / `emit_default_return` interleaving (mirroring
        // `emit_inner`), which a plain block descent does not express. The
        // constructor inline path has no such epilogue and so routes its stage
        // blocks through `Block::emit`.
        let mut current_block = entry;
        let mut terminated = false;
        for statement in modifier_body.statements().iter() {
            match statement.emit(&mut emitter, current_block) {
                Some(next) => current_block = next,
                None => {
                    terminated = true;
                    break;
                }
            }
        }
        if !terminated {
            self.emit_default_return(scope, result_types, return_slots.as_slice(), &current_block);
        }
    }

    fn build_modifier_stages<'state, 'context, 'env>(
        &self,
        scope: &FunctionScope<'state, 'context>,
        environment: &Environment<'context, 'env>,
        mut block: BlockRef<'context, 'env>,
    ) -> (
        Vec<Block>,
        Vec<ModifierStageParams<'context, 'env>>,
        BlockRef<'context, 'env>,
    ) {
        let mut modifier_stages: Vec<Block> = Vec::new();
        let mut modifier_params: Vec<ModifierStageParams<'context, 'env>> = Vec::new();
        for invocation in self.modifier_invocations().iter() {
            // Resolve the invocation lexically (a plain `m`), else by its final
            // path segment (a namespace-qualified `M.C.m`); a base-constructor
            // invocation `Base(args)` resolves to neither and is skipped here.
            let resolved_modifier = match invocation.name().resolve_to_definition() {
                Some(Definition::Modifier(modifier)) => modifier,
                _ => match scope
                    .contract
                    .and_then(|contract| contract.resolve_qualified_modifier(&invocation))
                {
                    Some(modifier) => modifier,
                    None => continue,
                },
            };
            // Re-dispatch to the most-derived override of this modifier (a
            // `virtual` modifier overridden in a derived contract); a non-virtual
            // or library modifier keeps its lexical resolution.
            let modifier_definition = scope
                .contract
                .and_then(|contract| {
                    contract.resolve_modifier_override(&invocation, &resolved_modifier)
                })
                .unwrap_or(resolved_modifier);
            let Some(modifier_body) = modifier_definition.body() else {
                continue;
            };
            let argument_expressions = invocation
                .arguments()
                .and_then(|argument_list| argument_list.positional_arguments())
                .unwrap_or_default(); // recut-lint-allow: fail01 — a modifier invocation may carry no arguments
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
                        scope.state,
                        environment,
                        scope.storage_layout,
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
                let parameter_type = AstType::parameter(&parameter, &scope.state.builder);
                let cast = value.cast(AstType::new(parameter_type), &scope.state.builder, &block);
                let pointer =
                    Pointer::stack_slot(AstType::new(parameter_type), &scope.state.builder, &block);
                pointer.store(cast, &scope.state.builder, &block);
                stage_params.push(ModifierParameterBinding {
                    declaration: parameter.node_id(),
                    pointer: pointer.into_mlir(),
                    element_type: parameter_type,
                });
            }
            modifier_stages.push(modifier_body);
            modifier_params.push(stage_params);
        }
        (modifier_stages, modifier_params, block)
    }
}
