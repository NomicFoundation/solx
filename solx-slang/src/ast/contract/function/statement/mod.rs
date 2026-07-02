//!
//! Statement lowering to MLIR operations.
//!

pub mod control_flow;
pub mod event;
pub mod revert;
pub mod try_statement;
pub mod variable_declaration;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
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

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::BreakOperation;
use solx_mlir::ods::sol::ContinueOperation;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_for_effect::EmitForEffect;
use crate::ast::emit::emit_statement::EmitStatement;
use crate::ast::emit::emit_values::EmitValues;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    pub environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks.
    /// Stored as a raw pointer to allow switching between Sol op regions
    /// without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `emit_return` to cast to.
    return_types: &'state [Type<'context>],
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default. Set to `false` inside `unchecked {}` blocks.
    pub checked: bool,
}

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
    ) -> Self {
        Self {
            state,
            environment,
            region_pointer: region as *const Region<'context>,
            storage_layout,
            return_types,
            checked: true,
        }
    }

    /// Borrows an [`ExpressionContext`] sharing this statement's scope.
    pub fn expression_context(&self) -> ExpressionContext<'_, 'context, 'block> {
        ExpressionContext::new(self.state, self.environment, self.storage_layout, self.checked)
    }

    /// Switches the current region for emitting into Sol op regions.
    pub fn set_region(&mut self, region: &Region<'context>) {
        self.region_pointer = region as *const Region<'context>;
    }

    /// Emits a sequence of statements inside a new lexical scope.
    pub fn emit_block(
        &mut self,
        statements: Statements,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        self.environment.enter_scope();
        let mut current = block;
        for statement in statements.iter() {
            match statement.emit(self, current) {
                Some(next) => current = next,
                None => {
                    self.environment.exit_scope();
                    return None;
                }
            }
        }
        self.environment.exit_scope();
        Some(current)
    }
}

impl<'context: 'block, 'block> EmitStatement<'context, 'block> for Statement {
    /// Dispatches a statement to its variant's emission.
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
            Statement::TryStatement(inner) => inner.emit(context, block),
            _ => unreachable!(
                "unsupported statement: {:?}",
                std::mem::discriminant(self)
            ),
        }
    }
}

statement_emit!(ExpressionStatement; |node, context, block| {
    let expression = node.expression();
    if let Expression::FunctionCallExpression(call) = &expression
        && let Expression::Identifier(identifier) = call.operand()
        && identifier.name() == revert::IDENTIFIER
    {
        return context.emit_revert_call(call, block);
    }
    Some(expression.emit_for_effect(&context.expression_context(), block))
});

statement_emit!(BreakStatement; |context, block| {
    mlir_op_void!(context.state, &block, BreakOperation);
    None
});

statement_emit!(ContinueStatement; |context, block| {
    mlir_op_void!(context.state, &block, ContinueOperation);
    None
});

statement_emit!(Block; |node, context, block| {
    context.emit_block(node.statements(), block)
});

statement_emit!(UncheckedBlock; |node, context, block| {
    let saved_checked = context.checked;
    context.checked = false;
    let result = context.emit_block(node.block().statements(), block);
    context.checked = saved_checked;
    result
});

statement_emit!(ReturnStatement; |node, context, block| {
    let Some(expression) = node.expression() else {
        mlir_op_void!(context.state, &block, ReturnOperation.operands(&[]));
        return None;
    };

    let expression_context = context.expression_context();
    if context.return_types.len() == 1 {
        let return_type = context.return_types[0];
        let BlockAnd { value, block } =
            expression.emit_as(return_type, &expression_context, block);
        mlir_op_void!(context.state, &block, ReturnOperation.operands(&[value]));
        return None;
    }

    let BlockAnd { value: values, block } = expression.emit_values(&expression_context, block);
    let cast_values: Vec<_> = values
        .into_iter()
        .zip(context.return_types.iter())
        .map(|(value, &return_type)| {
            TypeConversion::from_target_type(return_type, context.state).emit(
                value,
                context.state,
                &block,
            )
        })
        .collect();

    mlir_op_void!(context.state, &block, ReturnOperation.operands(&cast_values));
    None
});
