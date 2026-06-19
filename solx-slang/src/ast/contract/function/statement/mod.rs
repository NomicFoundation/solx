//!
//! Statement emission to MLIR operations.
//!

use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod assembly;
pub mod control_flow;
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
use slang_solidity_v2::ast::UncheckedBlock;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::BreakOperation;
use solx_mlir::ods::sol::ContinueOperation;
use solx_mlir::ods::sol::ReturnOperation;

use self::expression_statement_kind::ExpressionStatementKind;
use self::modifier_strategy::ModifierStrategy;
use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::EmitStatement;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::storage_layout::StorageSlot;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    pub environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks. A raw pointer to allow
    /// switching between Sol op regions without lifetime conflicts; re-pointed
    /// by direct assignment (`context.region_pointer = &region as *const _`).
    pub region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `emit_return` to cast to.
    pub return_types: &'state [Type<'context>],
    /// The function's return slots, parallel to `return_types` (`None` for an
    /// unnamed return). A bare `return;` and the fall-through epilogue load these
    /// so the `sol.return` arity matches the declared returns.
    pub return_slots: &'state [Option<Value<'context, 'block>>],
    /// How the `_;` placeholder lowers: a regular function's body-call hand-off,
    /// a constructor's inline modifier chain, or nothing outside a modifier.
    /// Set by direct assignment (`context.modifier_strategy = …`).
    pub modifier_strategy: ModifierStrategy<'context, 'block>,
    /// Arithmetic overflow-checking mode for binary operations.
    ///
    /// [`ArithmeticMode::Checked`] by default; [`ArithmeticMode::Unchecked`]
    /// inside `unchecked {}` blocks.
    pub arithmetic_mode: ArithmeticMode,
}

/// Builds an [`ExpressionContext`] from a statement context. The unchecked
/// loop-step is the one site that builds its context explicitly instead, with
/// [`ArithmeticMode::Unchecked`].
impl<'state, 'context, 'block> From<&'state StatementContext<'_, 'context, 'block>>
    for ExpressionContext<'state, 'context, 'block>
{
    fn from(statement: &'state StatementContext<'_, 'context, 'block>) -> Self {
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
        }
    }

    /// Emits the inline modifier chain for a constructor body from the current
    /// stage. Stage 0 is the outermost modifier body; each `_;` recurses to the
    /// next stage, and the constructor body (final stage) runs at the innermost
    /// `_;`. A stage's parameters are bound in a scope bracketing the whole stage
    /// — including the `_;` tail — so a repeated modifier keeps a distinct binding
    /// per use. A constructor has no return value, so the chain unwinds past the
    /// last stage (no separate body call).
    pub fn emit_inline_modifier_chain(
        &mut self,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        let ModifierStrategy::InlineChain {
            stages,
            parameters,
            index,
        } = &self.modifier_strategy
        else {
            return Some(block);
        };
        let stage = *index;
        let Some(stage_block) = stages.get(stage).cloned() else {
            return Some(block);
        };
        let params = parameters.get(stage).cloned().unwrap_or_default(); // recut-lint-allow: fail01 — a modifier stage may declare no parameters
        // Advance the cursor for the recursive `_;` (the borrow of the strategy
        // ended once `stage_block` / `params` were cloned out), restore it after.
        if let ModifierStrategy::InlineChain { index, .. } = &mut self.modifier_strategy {
            *index = stage + 1;
        }
        // The stage's parameters bracket the whole stage — including the `_;`
        // tail — in their own scope; the stage block opens its own inner scope.
        self.environment.enter_scope();
        for binding in params {
            self.environment
                .define_variable(binding.declaration, binding.pointer);
        }
        let result = stage_block.emit(self, block);
        self.environment.exit_scope();
        if let ModifierStrategy::InlineChain { index, .. } = &mut self.modifier_strategy {
            *index = stage;
        }
        result
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
        let builder = &context.state.builder;
        let mut values = Vec::with_capacity(context.return_types.len());
        for (index, &return_type) in context.return_types.iter().enumerate() {
            let value = match context.return_slots.get(index).copied().flatten() {
                Some(pointer) => Pointer::new(pointer)
                    .load(AstType::new(return_type), builder, &block)
                    .into_mlir(),
                None => AstValue::constant(
                    0,
                    AstType::new(return_type),
                    builder,
                    &block,
                )
                .into_mlir(),
            };
            values.push(value);
        }
        mlir_op_void!(builder, &block, ReturnOperation.operands(&values));
        return None;
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
                .expect("slang validated");
            let BlockAnd { value, block: next } = inner.emit(&emitter, current);
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
                call.emit(&emitter, block)
            }
            Expression::ConditionalExpression(conditional) => {
                conditional.emit(&emitter, block)
            }
            _ => {
                unimplemented!("multi-value return of a non-call expression is not supported")
            }
        };
        (
            values.into_iter().map(AstValue::from).collect(),
            block,
        )
    } else {
        // A single-value return materialises a string literal toward the
        // declared return type (a `bytesN`/`byte` constant), not a runtime
        // string the cast below would reject.
        let return_type = context.return_types[0];
        let BlockAnd { value, block } =
            if let Expression::StringExpression(string_literal) = &expression {
                string_literal.emit_as(return_type, &emitter, block)
            } else {
                expression.emit(&emitter, block)
            };
        (vec![value], block)
    };

    let cast_values: Vec<_> = values
        .into_iter()
        .zip(context.return_types.iter())
        .map(|(value, &return_type)| {
            value
                .cast(
                    AstType::new(return_type),
                    &context.state.builder,
                    &block,
                )
                .into_mlir()
        })
        .collect();

    mlir_op_void!(
        &context.state.builder,
        &block,
        ReturnOperation.operands(&cast_values)
    );
    None
});

statement_emit!(Block; |node, context, block| {
    // A `{ … }` block emits its statements in a fresh lexical scope, threading the
    // continuation block and short-circuiting the moment control diverges.
    context.environment.enter_scope();
    let result = node
        .statements()
        .iter()
        .try_fold(block, |block, statement| statement.emit(context, block));
    context.environment.exit_scope();
    result
});

statement_emit!(BreakStatement; |context, block| {
    mlir_op_void!(&context.state.builder, &block, BreakOperation);
    None
});

statement_emit!(ContinueStatement; |context, block| {
    mlir_op_void!(&context.state.builder, &block, ContinueOperation);
    None
});

// A bare expression statement discards its value but keeps its side effects.
// The shape is classified once ([`ExpressionStatementKind`]) and emitted by
// kind: the modifier `_;` placeholder hands off to the wrapped body / next
// stage, a `revert(...)` call diverges, a value-less type / `super` reference
// emits nothing, a type reference runs only its subexpressions' side effects, a
// tuple-valued conditional routes through the tuple path, and any other
// expression is emitted and its value discarded.
statement_emit!(ExpressionStatement; |node, context, block| {
    match ExpressionStatementKind::from_statement(node) {
        // A constructor's modifiers run as an inline chain, where `_;`
        // recurses to the next stage / the constructor body; a regular
        // function's modifiers are separate `sol.func`s, where `_;` hands off
        // through the body call.
        ExpressionStatementKind::ModifierPlaceholder => {
            if matches!(context.modifier_strategy, ModifierStrategy::InlineChain { .. }) {
                context.emit_inline_modifier_chain(block)
            } else {
                // A regular function's `_;` calls the wrapped body / next stage.
                if let ModifierStrategy::BodyCall(body_call) = &context.modifier_strategy {
                    body_call.emit(&context.state.builder, &block);
                }
                Some(block)
            }
        }
        ExpressionStatementKind::RevertCall(call) => context.emit_revert_call(&call, block),
        ExpressionStatementKind::TypeOrSuperNoop => Some(block),
        ExpressionStatementKind::TypeReference(expression) => {
            // A discarded type / selector reference (`C.f.selector;`) is a
            // compile-time value; only a runtime receiver buried under the
            // member-access chain — a conditional, `(c ? a : b).f.selector` —
            // carries side effects, so peel to the base and run just that.
            let mut current = expression.unwrap_parentheses();
            while let Expression::MemberAccessExpression(access) = current {
                current = access.operand().unwrap_parentheses();
            }
            if let Expression::ConditionalExpression(conditional) = current {
                let emitter = ExpressionContext::from(&*context);
                Some(conditional.operand().emit(&emitter, block).block)
            } else {
                Some(block)
            }
        }
        ExpressionStatementKind::TupleConditional(conditional) => {
            let emitter = ExpressionContext::from(&*context);
            let (_values, block) = conditional.emit(&emitter, block);
            Some(block)
        }
        ExpressionStatementKind::Value(expression) => {
            let emitter = ExpressionContext::from(&*context);
            Some(expression.emit_for_effect(&emitter, block))
        }
    }
});

statement_emit!(UncheckedBlock; |node, context, block| {
    let saved_mode = context.arithmetic_mode;
    context.arithmetic_mode = ArithmeticMode::Unchecked;
    let result = node.block().emit(context, block);
    context.arithmetic_mode = saved_mode;
    result
});

impl<'context: 'block, 'block> EmitStatement<'context, 'block> for Statement {
    /// Dispatches a statement to its variant's emission, threading the
    /// continuation block (`None` when control diverged).
    fn emit<'state>(
        &self,
        context: &mut StatementContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
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
