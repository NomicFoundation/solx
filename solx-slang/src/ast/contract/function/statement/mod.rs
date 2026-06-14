//!
//! Statement lowering to MLIR operations.
//!

pub mod assembly;
pub mod control_flow;
pub mod discarded;
pub mod event;
pub mod expression_statement_kind;
pub mod modifier_strategy;
pub mod revert;
pub mod try_statement;
pub mod variable_declaration;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::Block;
use slang_solidity_v2::ast::BreakStatement;
use slang_solidity_v2::ast::ContinueStatement;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ExpressionStatement;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::ReturnStatement;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::UncheckedBlock;
use slang_solidity_v2::ast::YulFunctionDefinition;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::BreakOperation;
use solx_mlir::ods::sol::ConditionOperation;
use solx_mlir::ods::sol::ContinueOperation;
use solx_mlir::ods::sol::ReturnOperation;

use self::discarded::Discarded;
use self::expression_statement_kind::ExpressionStatementKind;
use self::modifier_strategy::ModifierStrategy;
use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::function::modifier_parameter_binding::ModifierParameterBinding;
use crate::ast::contract::storage_layout::StorageSlot;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks.
    /// Stored as a raw pointer to allow switching between Sol op regions
    /// without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `emit_return` to cast to.
    return_types: &'state [Type<'context>],
    /// The function's return slots, parallel to `return_types` (`None` for an
    /// unnamed return). A bare `return;` and the fall-through epilogue load these
    /// so the `sol.return` arity matches the declared returns.
    return_slots: &'state [Option<Value<'context, 'block>>],
    /// How the `_;` placeholder lowers: a regular function's body-call hand-off,
    /// a constructor's inline modifier chain, or nothing outside a modifier.
    modifier_strategy: ModifierStrategy<'context, 'block>,
    /// Arithmetic overflow-checking mode for binary operations.
    ///
    /// [`ArithmeticMode::Checked`] by default; [`ArithmeticMode::Unchecked`]
    /// inside `unchecked {}` blocks.
    arithmetic_mode: ArithmeticMode,
    /// User-defined Yul functions in scope within an `assembly { … }` block,
    /// keyed by name. Each is inlined at its call sites; an entry lives only
    /// for the duration of the Yul block (or inlined function frame) that
    /// declares it, then is removed so outer-scope definitions remain.
    yul_functions: HashMap<String, YulFunctionDefinition>,
    /// Per-name inline-recursion guard: a Yul function currently being inlined
    /// has depth ≥ 1, so a recursive call is rejected (it would otherwise loop
    /// the compiler) rather than emitted.
    yul_inline_depth: HashMap<String, usize>,
}

/// Builds an [`ExpressionContext`] for a statement context — the shared state,
/// environment, storage layout, and arithmetic mode every expression emission
/// needs. The unchecked loop-step is the one site that builds its context
/// explicitly instead, with [`ArithmeticMode::Unchecked`].
impl<'a, 'context, 'block> From<&'a StatementContext<'_, 'context, 'block>>
    for ExpressionContext<'a, 'context, 'block>
{
    fn from(statement: &'a StatementContext<'_, 'context, 'block>) -> Self {
        ExpressionContext::new(
            statement.state,
            statement.environment,
            statement.storage_layout,
            statement.arithmetic_mode,
        )
    }
}

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
        return_slots: &'state [Option<Value<'context, 'block>>],
    ) -> Self {
        Self {
            state,
            environment,
            region_pointer: region as *const Region<'context>,
            storage_layout,
            return_types,
            return_slots,
            modifier_strategy: ModifierStrategy::None,
            arithmetic_mode: ArithmeticMode::Checked,
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
        }
    }

    /// Sets the modifier-stage hand-off the `_;` placeholder lowers to. Called by
    /// [`FunctionEmitter::emit_modifier_stage_func`] before threading a modifier
    /// body's statements.
    ///
    /// [`FunctionEmitter::emit_modifier_stage_func`]: crate::ast::contract::function::FunctionEmitter::emit_modifier_stage_func
    pub fn set_modifier_body_call(&mut self, call: ModifierBodyCall<'context, 'block>) {
        self.modifier_strategy = ModifierStrategy::BodyCall(call);
    }

    /// Loads the inline modifier-chain stages for a constructor body emission
    /// (built by [`FunctionEmitter::build_modifier_stages`], with the constructor
    /// body pushed as the final stage), then drive them with
    /// [`Self::emit_inline_modifier_chain`].
    pub fn set_modifier_stages(
        &mut self,
        modifier_stages: Vec<Statements>,
        modifier_stage_params: Vec<Vec<ModifierParameterBinding<'context, 'block>>>,
    ) {
        self.modifier_strategy = ModifierStrategy::InlineChain {
            stages: modifier_stages,
            parameters: modifier_stage_params,
            index: 0,
        };
    }

    /// Emits the inline modifier chain for a constructor body from the current
    /// stage. Stage 0 is the outermost modifier body; each `_;` placeholder
    /// recurses to the next stage, and the constructor body (the final stage)
    /// runs at the innermost `_;`. Each stage's modifier parameters are bound in a
    /// scope that brackets the whole stage — including the `_;` tail — so a
    /// repeated modifier keeps a distinct binding per use and the binding is gone
    /// once the stage unwinds. A constructor has no return value, so the chain
    /// simply unwinds past the last stage (no separate body call).
    pub fn emit_inline_modifier_chain(
        &mut self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let ModifierStrategy::InlineChain {
            stages,
            parameters,
            index,
        } = &self.modifier_strategy
        else {
            return Ok(Some(block));
        };
        let stage = *index;
        let Some(statements) = stages.get(stage).cloned() else {
            return Ok(Some(block));
        };
        let params = parameters.get(stage).cloned().unwrap_or_default();
        // Advance the cursor for the recursive `_;` (the borrow of the strategy
        // ended once `statements` / `params` were cloned out), restore it after.
        self.set_modifier_stage_index(stage + 1);
        self.environment.enter_scope();
        for binding in params {
            self.environment
                .define_variable(binding.declaration, binding.pointer);
        }
        let result = self.emit_block(statements, block);
        self.environment.exit_scope();
        self.set_modifier_stage_index(stage);
        result
    }

    /// Sets the inline modifier-chain cursor. A no-op unless an
    /// [`ModifierStrategy::InlineChain`] is active.
    fn set_modifier_stage_index(&mut self, stage: usize) {
        if let ModifierStrategy::InlineChain { index, .. } = &mut self.modifier_strategy {
            *index = stage;
        }
    }

    /// Returns a reference to the current region.
    ///
    /// # Safety
    ///
    /// The region pointer is valid as long as the MLIR module exists,
    /// which outlives all emitters.
    pub fn region(&self) -> &Region<'context> {
        // SAFETY: The region is owned by the MLIR module and outlives this emitter.
        unsafe { &*self.region_pointer }
    }

    /// Switches the current region for emitting into Sol op regions.
    pub fn set_region(&mut self, region: &Region<'context>) {
        self.region_pointer = region as *const Region<'context>;
    }

    /// Evaluates a loop `condition` in `condition_block` and emits the
    /// `sol.condition` terminator. Shared by `for`, `while`, and `do-while`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition expression contains unsupported
    /// constructs.
    fn emit_loop_condition(
        &self,
        condition: &Expression,
        condition_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let emitter = ExpressionContext::from(self);
        let BlockAnd {
            value: condition_value,
            block: condition_end,
        } = condition.emit(&emitter, condition_block)?;
        let condition_boolean = condition_value
            .is_nonzero(&self.state.builder, &condition_end)
            .into_mlir();
        sol_op_void!(
            &self.state.builder,
            &condition_end,
            ConditionOperation.condition(condition_boolean)
        );
        Ok(())
    }

    /// Emits a sequence of statements inside a new lexical scope.
    ///
    /// # Errors
    ///
    /// Returns an error if any statement contains unsupported constructs.
    pub fn emit_block(
        &mut self,
        statements: Statements,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();
        let mut current = block;
        for statement in statements.iter() {
            match statement.emit(self, current)? {
                Some(next) => current = next,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            }
        }
        self.environment.exit_scope();
        Ok(Some(current))
    }

    /// Emits only the side effects of a discarded type-reference expression: a
    /// member access recurses into its operand; a conditional whose branches are
    /// types runs only its condition (the type branches are compile-time, with no
    /// runtime value or side effect); any other type/module reference has none.
    fn emit_type_reference_side_effects(
        &mut self,
        expression: Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match expression.unwrap_parentheses() {
            Expression::MemberAccessExpression(access) => {
                self.emit_type_reference_side_effects(access.operand(), block)
            }
            Expression::ConditionalExpression(conditional) => {
                let emitter = ExpressionContext::from(&*self);
                let operand = conditional.operand();
                let BlockAnd { block, .. } = operand.emit(&emitter, block)?;
                Ok(Some(block))
            }
            _ => Ok(Some(block)),
        }
    }

    /// Lowers a modifier stage's `_;` placeholder to the modifier-body hand-off
    /// (call the wrapped body / next stage, threading the shared return values),
    /// delegating to [`ModifierBodyCall::emit`]. Outside a modifier stage `_;`
    /// has no hand-off set and emits nothing.
    ///
    /// # Errors
    ///
    /// Returns an error if the hand-off call cannot be lowered.
    fn emit_modifier_body_call(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        if let ModifierStrategy::BodyCall(body_call) = &self.modifier_strategy {
            body_call.emit(&self.state.builder, &block)?;
        }
        Ok(Some(block))
    }
}

// A multi-element tuple expression in the return position is unpacked into one
// value per declared return slot; any other expression yields a single value.
// Each value is cast to its declared return type before the `sol.return`.
statement_emit!(ReturnStatement; |node, context, block| {
    let Some(expression) = node.expression() else {
        // A bare `return;` returns the current values of the return slots
        // (zero for an unnamed/unset slot), so its `sol.return` arity matches
        // the enclosing function — like the fall-through epilogue. This
        // matters for a `return;` in a modifier stage of a value-returning
        // function, where a 0-operand return would fail verification; a void
        // function has no slots and returns nothing.
        context
            .state
            .builder
            .emit_return_from_slots(context.return_types, context.return_slots, &block);
        return Ok(None);
    };

    let emitter = ExpressionContext::from(&*context);

    let (values, block) = if let Expression::TupleExpression(tuple) = &expression
        && tuple.items().len() > 1
    {
        let items = tuple.items();
        let mut values = Vec::with_capacity(items.len());
        let mut current = block;
        for item in items.iter() {
            let inner = item
                .expression()
                .expect("a return tuple element has an inner expression");
            let BlockAnd { value, block: next } = inner.emit(&emitter, current)?;
            values.push(value);
            current = next;
        }
        (values, current)
    } else if context.return_types.len() > 1 {
        // A single expression that yields multiple values is either a
        // tuple-returning call (`return f();`), where solc emits one
        // `sol.call` with N results, or a conditional with tuple branches
        // (`return cond ? (1, 2) : (3, 4);`). Expand its full result list so
        // the `sol.return` arity matches rather than taking the first value.
        let (values, block) = match &expression {
            Expression::FunctionCallExpression(call) => {
                emitter.emit_function_call_results(call, block)?
            }
            Expression::ConditionalExpression(conditional) => {
                emitter.emit_conditional_tuple_values(conditional, block)?
            }
            _ => {
                unimplemented!("multi-value return of a non-call expression is not supported")
            }
        };
        (
            values.into_iter().map(crate::ast::Value::from).collect(),
            block,
        )
    } else {
        // A single-value return materialises a string literal toward the
        // declared return type (a `bytesN`/`byte` constant), not a runtime
        // string the cast below would reject.
        let return_type = context.return_types[0];
        let BlockAnd { value, block } = (Toward {
            expression: &expression,
            target_type: return_type,
        })
        .emit(&emitter, block)?;
        (vec![value], block)
    };

    let cast_values: Vec<_> = values
        .into_iter()
        .zip(context.return_types.iter())
        .map(|(value, &return_type)| {
            value
                .coerce_to(
                    crate::ast::Type::new(return_type),
                    &context.state.builder,
                    &block,
                )
                .into_mlir()
        })
        .collect();

    sol_op_void!(
        &context.state.builder,
        &block,
        ReturnOperation.operands(&cast_values)
    );
    Ok(None)
});

statement_emit!(Block; |node, context, block| {
    context.emit_block(node.statements(), block)
});

statement_emit!(BreakStatement; |context, block| {
    sol_op_void!(&context.state.builder, &block, BreakOperation);
    Ok(None)
});

statement_emit!(ContinueStatement; |context, block| {
    sol_op_void!(&context.state.builder, &block, ContinueOperation);
    Ok(None)
});

// A bare expression statement discards its value but keeps its side effects.
// The shape is classified once ([`ExpressionStatementKind`]) and emitted by
// kind: the modifier `_;` placeholder hands off to the wrapped body / next
// stage, a `revert(...)` call diverges, a value-less type / `super` reference
// emits nothing, a type reference runs only its subexpressions' side effects, a
// tuple-valued conditional routes through the tuple path, and any other
// expression is emitted and its value discarded.
statement_emit!(ExpressionStatement; |node, context, block| {
    match ExpressionStatementKind::classify(node) {
        // A constructor's modifiers run as an inline chain, where `_;`
        // recurses to the next stage / the constructor body; a regular
        // function's modifiers are separate `sol.func`s, where `_;` hands off
        // through the body call.
        ExpressionStatementKind::ModifierPlaceholder => {
            if matches!(context.modifier_strategy, ModifierStrategy::InlineChain { .. }) {
                context.emit_inline_modifier_chain(block)
            } else {
                context.emit_modifier_body_call(block)
            }
        }
        ExpressionStatementKind::RevertCall(call) => context.emit_revert_call(&call, block),
        ExpressionStatementKind::TypeOrSuperNoop => Ok(Some(block)),
        ExpressionStatementKind::TypeReference(expression) => {
            context.emit_type_reference_side_effects(expression, block)
        }
        ExpressionStatementKind::TupleConditional(conditional) => {
            let emitter = ExpressionContext::from(&*context);
            let (_values, block) = emitter.emit_conditional_tuple_values(&conditional, block)?;
            Ok(Some(block))
        }
        ExpressionStatementKind::Value(expression) => {
            let emitter = ExpressionContext::from(&*context);
            Ok(Some(Discarded(&expression).emit(&emitter, block)?))
        }
    }
});

statement_emit!(UncheckedBlock; |node, context, block| {
    let saved_mode = context.arithmetic_mode;
    context.arithmetic_mode = ArithmeticMode::Unchecked;
    let result = context.emit_block(node.block().statements(), block);
    context.arithmetic_mode = saved_mode;
    result
});

impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope> for Statement
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope mut StatementContext<'state, 'context, 'block>;
    type Output = Option<BlockRef<'context, 'block>>;

    /// Dispatches a statement to its variant's lowering, threading the
    /// continuation block (`None` when control diverged).
    fn emit(
        &self,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Self::Output> {
        match self {
            Statement::VariableDeclarationStatement(inner) => inner.emit(context, block),
            Statement::ExpressionStatement(inner) => inner.emit(context, block),
            Statement::ReturnStatement(inner) => inner.emit(context, block),
            Statement::IfStatement(inner) => inner.emit(context, block),
            Statement::ForStatement(inner) => inner.emit(context, block),
            Statement::WhileStatement(inner) => inner.emit(context, block),
            Statement::DoWhileStatement(inner) => inner.emit(context, block),
            Statement::BreakStatement(inner) => inner.emit(context, block),
            Statement::ContinueStatement(inner) => inner.emit(context, block),
            Statement::Block(inner) => inner.emit(context, block),
            Statement::UncheckedBlock(inner) => inner.emit(context, block),
            Statement::RevertStatement(inner) => inner.emit(context, block),
            Statement::EmitStatement(inner) => inner.emit(context, block),
            Statement::AssemblyStatement(inner) => inner.emit(context, block),
            Statement::TryStatement(inner) => inner.emit(context, block),
        }
    }
}
