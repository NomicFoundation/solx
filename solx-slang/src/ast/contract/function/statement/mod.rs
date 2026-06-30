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
use solx_mlir::ods::sol::PlaceholderOperation;
use solx_mlir::ods::sol::ReturnOperation;
use solx_mlir::ods::sol::RevertOperation;

use self::expression_statement_kind::ExpressionStatementKind;
use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::EmitStatement;
use crate::ast::EmitValues;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::storage_layout::StorageSlot;

/// Emits Solidity statements to MLIR operations; threads `Some(block)` or `None` when control diverges.
pub struct StatementContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment, mutable for new declarations and loop targets.
    pub environment: &'state mut Environment<'context, 'block>,
    /// Contract-local dispatch metadata.
    pub dispatch: &'state ContractDispatch,
    /// The current region for new blocks. A raw pointer to switch between Sol op regions without
    /// lifetime conflicts; re-pointed by direct assignment.
    pub region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `emit_return` to cast to.
    pub return_types: &'state [Type<'context>],
    /// The function's return slots, parallel to `return_types` (`None` for an unnamed return); a bare
    /// `return;` and the epilogue load these so the `sol.return` arity matches.
    pub return_slots: &'state [Option<Value<'context, 'block>>],
    /// Arithmetic overflow-checking mode: Checked by default, Unchecked inside `unchecked {}`.
    pub arithmetic_mode: ArithmeticMode,
}

/// Builds an [`ExpressionContext`] from a statement context (propagating its arithmetic mode).
impl<'state, 'context, 'block> From<&'state StatementContext<'_, 'context, 'block>>
    for ExpressionContext<'state, 'context, 'block>
{
    fn from(statement: &'state StatementContext<'_, 'context, 'block>) -> Self {
        ExpressionContext::new(
            statement.state,
            statement.environment,
            statement.dispatch,
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
        dispatch: &'state ContractDispatch,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
        return_slots: &'state [Option<Value<'context, 'block>>],
    ) -> Self {
        Self {
            state,
            environment,
            dispatch,
            region_pointer: region as *const Region<'context>,
            storage_layout,
            return_types,
            return_slots,
            arithmetic_mode: ArithmeticMode::Checked,
        }
    }
}

statement_emit!(ReturnStatement; |node, context, block| {
    let Some(expression) = node.expression() else {
        let state = context.state;
        let mut values = Vec::with_capacity(context.return_types.len());
        for (index, &return_type) in context.return_types.iter().enumerate() {
            let value = match context.return_slots[index] {
                Some(pointer) => Pointer::new(pointer)
                    .load(AstType::new(return_type), state, &block)
                    .into_mlir(),
                None => AstValue::constant(
                    0,
                    AstType::new(return_type),
                    state,
                    &block,
                )
                .into_mlir(),
            };
            values.push(value);
        }
        mlir_op_void!(state, &block, ReturnOperation.operands(&values));
        return None;
    };

    let emitter = ExpressionContext::from(&*context);
    let expression = expression.unwrap_parentheses();

    let (values, block) = if context.return_types.len() > 1 {
        let BlockAnd { value: values, block } = expression.emit_values(&emitter, block);
        (values.into_iter().map(AstValue::from).collect(), block)
    } else {
        let return_type = context.return_types[0];
        let BlockAnd { value, block } = expression.emit_as(return_type, &emitter, block);
        (vec![value], block)
    };

    let cast_values: Vec<_> = values
        .into_iter()
        .zip(context.return_types.iter())
        .map(|(value, &return_type)| {
            value
                .cast(
                    AstType::new(return_type),
                    context.state,
                    &block,
                )
                .into_mlir()
        })
        .collect();

    mlir_op_void!(
        context.state,
        &block,
        ReturnOperation.operands(&cast_values)
    );
    None
});

statement_emit!(Block; |node, context, block| {
    context.environment.enter_scope();
    let result = node
        .statements()
        .iter()
        .try_fold(block, |block, statement| statement.emit(context, block));
    context.environment.exit_scope();
    result
});

statement_emit!(BreakStatement; |context, block| {
    mlir_op_void!(context.state, &block, BreakOperation);
    None
});

statement_emit!(ContinueStatement; |context, block| {
    mlir_op_void!(context.state, &block, ContinueOperation);
    None
});

statement_emit!(ExpressionStatement; |node, context, block| {
    match ExpressionStatementKind::from_statement(node) {
        ExpressionStatementKind::ModifierPlaceholder => {
            mlir_op_void!(context.state, &block, PlaceholderOperation);
            Some(block)
        }
        ExpressionStatementKind::RevertCall(call) => {
            let argument = match &call.arguments() {
                ArgumentsDeclaration::PositionalArguments(positional_arguments) => {
                    positional_arguments.iter().next()
                }
                ArgumentsDeclaration::NamedArguments(named_arguments)
                    if named_arguments.iter().next().is_none() =>
                {
                    None
                }
                ArgumentsDeclaration::NamedArguments(_) => {
                    unreachable!("named arguments on a revert are not supported");
                }
            };
            let block = match argument {
                None => {
                    let state = context.state;
                    mlir_op_void!(
                        state,
                        &block,
                        RevertOperation
                            .signature(StringAttribute::new(state.mlir_context, ""))
                            .args(&[])
                    );
                    block
                }
                Some(Expression::StringExpression(string_expression))
                    if !string_expression.value().is_empty() =>
                {
                    let message = String::from_utf8(string_expression.value())
                        .expect("revert message is valid UTF-8");
                    let state = context.state;
                    mlir_op_void!(
                        state,
                        &block,
                        RevertOperation
                            .signature(StringAttribute::new(state.mlir_context, &message))
                            .args(&[])
                    );
                    block
                }
                Some(expression) => {
                    let emitter = ExpressionContext::from(&*context);
                    let BlockAnd {
                        value: message_value,
                        block,
                    } = expression.emit(&emitter, block);
                    let state = context.state;
                    let string_memory_type =
                        AstType::string(state.mlir_context, solx_utils::DataLocation::Memory)
                            .into_mlir();
                    let message_value = message_value
                        .cast(AstType::new(string_memory_type), state, &block)
                        .into_mlir();
                    mlir_op_void!(
                        state,
                        &block,
                        RevertOperation
                            .signature(StringAttribute::new(state.mlir_context, "Error(string)"))
                            .args(&[message_value])
                            .call(Attribute::unit(state.mlir_context))
                    );
                    block
                }
            };
            Some(block)
        }
        ExpressionStatementKind::TypeOrSuperNoop => Some(block),
        ExpressionStatementKind::TypeReference(expression) => {
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
