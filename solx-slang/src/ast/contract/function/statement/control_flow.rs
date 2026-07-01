//!
//! Control flow statement lowering: if/else, for, while, do-while.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use slang_solidity_v2::ast::DoWhileStatement;
use slang_solidity_v2::ast::ForStatement;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::IfStatement;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::WhileStatement;

use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ConditionOperation;
use solx_mlir::ods::sol::DoWhileOperation;
use solx_mlir::ods::sol::ForOperation;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::WhileOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_for_effect::EmitForEffect;
use crate::ast::emit::emit_statement::EmitStatement;

statement_emit!(IfStatement; |node, context, block| {
    let condition_expression = node.condition();
    let expression_context = context.expression_context();
    let BlockAnd {
        value: condition_value,
        block,
    } = condition_expression.emit(&expression_context, block);
    let condition_boolean = expression_context.emit_is_nonzero(condition_value, &block);

    let (then_block, else_block) = mlir_region_op!(
        context.state, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let then_region = solx_mlir::ffi::block_parent_region(&then_block);
    let else_region = solx_mlir::ffi::block_parent_region(&else_block);

    let saved_region = context.region_pointer;
    context.set_region(&then_region);
    let then_end = node.body().emit(context, then_block);
    if let Some(then_end) = then_end {
        mlir_op_void!(context.state, &then_end, YieldOperation.ins(&[]));
    } else {
        context.emit_dead_yield(&then_region);
    }

    if let Some(ref else_statement) = node.else_branch() {
        context.set_region(&else_region);
        let else_end = else_statement.emit(context, else_block);
        if let Some(else_end) = else_end {
            mlir_op_void!(context.state, &else_end, YieldOperation.ins(&[]));
        } else {
            context.emit_dead_yield(&else_region);
        }
    } else {
        mlir_op_void!(context.state, &else_block, YieldOperation.ins(&[]));
    }
    context.region_pointer = saved_region;

    Some(block)
});

statement_emit!(ForStatement; |node, context, block| {
    context.environment.enter_scope();

    let block = match node.initialization() {
        ForStatementInitialization::VariableDeclarationStatement(declaration) => {
            match Statement::VariableDeclarationStatement(declaration.clone()).emit(context, block) {
                Some(block) => block,
                None => {
                    context.environment.exit_scope();
                    return None;
                }
            }
        }
        ForStatementInitialization::ExpressionStatement(expression_statement) => {
            match Statement::ExpressionStatement(expression_statement.clone()).emit(context, block) {
                Some(block) => block,
                None => {
                    context.environment.exit_scope();
                    return None;
                }
            }
        }
        ForStatementInitialization::Semicolon(_) => block,
    };

    let (condition_block, body_block, step_block) =
        mlir_region_op!(context.state, &block, ForOperation; cond, body, step);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    match node.condition() {
        ForStatementCondition::ExpressionStatement(expression_statement) => {
            let expression = expression_statement.expression();
            let expression_context = context.expression_context();
            let BlockAnd {
                value: condition_value,
                block: condition_end,
            } = expression.emit(&expression_context, condition_block);
            let condition_boolean =
                expression_context.emit_is_nonzero(condition_value, &condition_end);
            mlir_op_void!(
                context.state,
                &condition_end,
                ConditionOperation.condition(condition_boolean)
            );
        }
        ForStatementCondition::Semicolon(_) => {
            let true_value =
                AstValue::boolean(true, context.state, &condition_block).into_mlir();
            mlir_op_void!(
                context.state,
                &condition_block,
                ConditionOperation.condition(true_value)
            );
        }
    }

    context.set_region(&body_region);
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        mlir_op_void!(context.state, &body_end, YieldOperation.ins(&[]));
    }

    if let Some(ref iterator_expression) = node.iterator() {
        let saved_checked = context.checked;
        context.checked = false;
        let step_end = iterator_expression.emit_for_effect(&context.expression_context(), step_block);
        context.checked = saved_checked;
        mlir_op_void!(context.state, &step_end, YieldOperation.ins(&[]));
    } else {
        mlir_op_void!(context.state, &step_block, YieldOperation.ins(&[]));
    }

    context.region_pointer = saved_region;
    context.environment.exit_scope();
    Some(block)
});

statement_emit!(WhileStatement; |node, context, block| {
    let (condition_block, body_block) =
        mlir_region_op!(context.state, &block, WhileOperation; cond, body);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    let condition_expression = node.condition();
    let expression_context = context.expression_context();
    let BlockAnd {
        value: condition_value,
        block: condition_end,
    } = condition_expression.emit(&expression_context, condition_block);
    let condition_boolean = expression_context.emit_is_nonzero(condition_value, &condition_end);
    mlir_op_void!(
        context.state,
        &condition_end,
        ConditionOperation.condition(condition_boolean)
    );

    context.set_region(&body_region);
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        mlir_op_void!(context.state, &body_end, YieldOperation.ins(&[]));
    }

    context.region_pointer = saved_region;
    Some(block)
});

statement_emit!(DoWhileStatement; |node, context, block| {
    let (body_block, condition_block) =
        mlir_region_op!(context.state, &block, DoWhileOperation; body, cond);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    context.set_region(&body_region);
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        mlir_op_void!(context.state, &body_end, YieldOperation.ins(&[]));
    }

    let condition_expression = node.condition();
    let expression_context = context.expression_context();
    let BlockAnd {
        value: condition_value,
        block: condition_end,
    } = condition_expression.emit(&expression_context, condition_block);
    let condition_boolean = expression_context.emit_is_nonzero(condition_value, &condition_end);
    mlir_op_void!(
        context.state,
        &condition_end,
        ConditionOperation.condition(condition_boolean)
    );

    context.region_pointer = saved_region;
    Some(block)
});

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Appends a dead block with `sol.yield` to a region whose live block
    /// already terminated (e.g. with `sol.return`). Matches the solc pattern
    /// where each `sol.if` region always ends with a `sol.yield` block.
    pub(super) fn emit_dead_yield(&self, region: &melior::ir::Region<'context>) {
        let dead_block = melior::ir::Block::new(&[]);
        mlir_op_void!(self.state, &dead_block, YieldOperation.ins(&[]));
        region.append_block(dead_block);
    }
}
