//!
//! Control flow statement emission: if/else, for, while, do-while.
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
use solx_mlir::ods::sol::ConditionOperation;
use solx_mlir::ods::sol::YieldOperation;

use super::discarded::Discarded;
use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::StatementContext;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Appends a dead block with `sol.yield` to a region whose live block
    /// already terminated (e.g. with `sol.return`). Matches the solc pattern
    /// where each `sol.if` region always ends with a `sol.yield` block.
    fn emit_dead_yield(&self, region: &melior::ir::Region<'context>) {
        let dead_block = melior::ir::Block::new(&[]);
        sol_op_void!(&self.state.builder, &dead_block, YieldOperation.ins(&[]));
        region.append_block(dead_block);
    }
}

statement_emit!(IfStatement; |node, context, block| {
    let condition_expression = node.condition();
    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: condition_value,
        block,
    } = condition_expression.emit(&emitter, block);
    let condition_boolean = condition_value
        .is_nonzero(&context.state.builder, &block)
        .into_mlir();

    let (then_block, else_block) = context.state.builder.emit_sol_if(condition_boolean, &block);

    // Get the inner regions for creating blocks in the right scope.
    let then_region = solx_mlir::ffi::block_parent_region(&then_block);
    let else_region = solx_mlir::ffi::block_parent_region(&else_block);

    // Emit then body.
    let saved_region = context.region_pointer;
    context.set_region(&then_region);
    let then_end = node.body().emit(context, then_block);
    if let Some(then_end) = then_end {
        sol_op_void!(&context.state.builder, &then_end, YieldOperation.ins(&[]));
    } else {
        context.emit_dead_yield(&then_region);
    }

    // Emit else body (or empty yield).
    if let Some(ref else_statement) = node.else_branch() {
        context.set_region(&else_region);
        let else_end = else_statement.emit(context, else_block);
        if let Some(else_end) = else_end {
            sol_op_void!(&context.state.builder, &else_end, YieldOperation.ins(&[]));
        } else {
            context.emit_dead_yield(&else_region);
        }
        context.region_pointer = saved_region;
    } else {
        sol_op_void!(&context.state.builder, &else_block, YieldOperation.ins(&[]));
        context.region_pointer = saved_region;
    }

    Some(block)
});

statement_emit!(ForStatement; |node, context, block| {
    context.environment.enter_scope();

    // Emit initialization in the current block.
    let block = match node.initialization() {
        ForStatementInitialization::VariableDeclarationStatement(declaration) => {
            let statement = Statement::VariableDeclarationStatement(declaration.clone());
            match statement.emit(context, block) {
                Some(block) => block,
                None => {
                    context.environment.exit_scope();
                    return None;
                }
            }
        }
        ForStatementInitialization::ExpressionStatement(expression_statement) => {
            let statement = Statement::ExpressionStatement(expression_statement.clone());
            match statement.emit(context, block) {
                Some(block) => block,
                None => {
                    context.environment.exit_scope();
                    return None;
                }
            }
        }
        ForStatementInitialization::Semicolon(_) => block,
    };

    let (condition_block, body_block, step_block) = context.state.builder.emit_sol_for(&block);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    // Condition region.
    match node.condition() {
        ForStatementCondition::ExpressionStatement(expression_statement) => {
            context.emit_loop_condition(&expression_statement.expression(), condition_block);
        }
        ForStatementCondition::Semicolon(_) => {
            let true_value =
                crate::ast::Value::boolean(true, &context.state.builder, &condition_block)
                    .into_mlir();
            sol_op_void!(
                &context.state.builder,
                &condition_block,
                ConditionOperation.condition(true_value)
            );
        }
    }

    // Body region.
    context.set_region(&body_region);
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        sol_op_void!(&context.state.builder, &body_end, YieldOperation.ins(&[]));
    }

    // Step region — always unchecked (matches solc: loop step i++ uses sol.add).
    if let Some(ref iterator_expression) = node.iterator() {
        let emitter = ExpressionContext::new(
            context.state,
            context.environment,
            context.storage_layout,
            // The loop step is always unchecked; solc emits `sol.add` for `i++`.
            ArithmeticMode::Unchecked,
        );
        // The step is in statement position: its value is discarded and it
        // may be a value-less producer (a void call or `delete`).
        let step_end = Discarded(iterator_expression).emit(&emitter, step_block);
        sol_op_void!(&context.state.builder, &step_end, YieldOperation.ins(&[]));
    } else {
        sol_op_void!(&context.state.builder, &step_block, YieldOperation.ins(&[]));
    }

    context.region_pointer = saved_region;
    context.environment.exit_scope();
    Some(block)
});

statement_emit!(WhileStatement; |node, context, block| {
    let (condition_block, body_block) = context.state.builder.emit_sol_while(&block);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    // Condition region.
    context.emit_loop_condition(&node.condition(), condition_block);

    // Body region.
    context.set_region(&body_region);
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        sol_op_void!(&context.state.builder, &body_end, YieldOperation.ins(&[]));
    }

    context.region_pointer = saved_region;
    Some(block)
});

statement_emit!(DoWhileStatement; |node, context, block| {
    let (body_block, condition_block) = context.state.builder.emit_sol_do_while(&block);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    // Body region (executes first).
    context.set_region(&body_region);
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        sol_op_void!(&context.state.builder, &body_end, YieldOperation.ins(&[]));
    }

    // Condition region.
    context.emit_loop_condition(&node.condition(), condition_block);

    context.region_pointer = saved_region;
    Some(block)
});
