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

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::ArgumentsDeclaration;
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
use solx_mlir::ods::sol::RevertOperation;

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
        let BlockAnd { value: values, block } = match &expression {
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
        // The placeholder hands off per the active modifier strategy: an inline
        // chain (a constructor) recurses to the next stage, a body call (a regular
        // function) calls the wrapped body / next stage `sol.func`.
        ExpressionStatementKind::ModifierPlaceholder => {
            ModifierStrategy::emit_placeholder(context, block)
        }
        ExpressionStatementKind::RevertCall(call) => {
            let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
            else {
                unimplemented!("only positional arguments supported");
            };
            // `sol.revert` is not a terminator; the block stays live for the
            // caller (an enclosing yield or the epilogue default return).
            let block = match positional_arguments.iter().next() {
                // `revert()` — a no-data revert.
                None => {
                    let builder = &context.state.builder;
                    mlir_op_void!(
                        builder,
                        &block,
                        RevertOperation
                            .signature(StringAttribute::new(builder.context, ""))
                            .args(&[])
                    );
                    block
                }
                // A non-empty string literal bakes the message into the op as the
                // `Error(string)` payload (no runtime encoding).
                Some(Expression::StringExpression(string_expression))
                    if !string_expression.value().is_empty() =>
                {
                    let message = String::from_utf8(string_expression.value())
                        .expect("revert message is valid UTF-8");
                    let builder = &context.state.builder;
                    mlir_op_void!(
                        builder,
                        &block,
                        RevertOperation
                            .signature(StringAttribute::new(builder.context, &message))
                            .args(&[])
                    );
                    block
                }
                // A non-literal message (`revert(expr)`) or an empty literal
                // (`revert("")`, i.e. `Error("")`) is evaluated at runtime and
                // ABI-encoded under the `Error(string)` selector, like
                // `require(cond, expr)`.
                Some(expression) => {
                    let emitter = ExpressionContext::from(&*context);
                    let BlockAnd {
                        value: message_value,
                        block,
                    } = expression.emit(&emitter, block);
                    let builder = &context.state.builder;
                    let string_memory_type =
                        AstType::string(builder.context, solx_utils::DataLocation::Memory)
                            .into_mlir();
                    let message_value = message_value
                        .cast(AstType::new(string_memory_type), builder, &block)
                        .into_mlir();
                    mlir_op_void!(
                        builder,
                        &block,
                        RevertOperation
                            .signature(StringAttribute::new(builder.context, "Error(string)"))
                            .args(&[message_value])
                            .call(Attribute::unit(builder.context))
                    );
                    block
                }
            };
            Some(block)
        }
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
            let BlockAnd { block, .. } = conditional.emit(&emitter, block);
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
