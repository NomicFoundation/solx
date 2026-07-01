//!
//! Control flow statement emission: if/else, for, while, do-while.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use melior::ir::RegionRef;
use slang_solidity_v2::ast::DoWhileStatement;
use slang_solidity_v2::ast::ForStatement;
use slang_solidity_v2::ast::ForStatementCondition;
use slang_solidity_v2::ast::ForStatementInitialization;
use slang_solidity_v2::ast::IfStatement;
use slang_solidity_v2::ast::WhileStatement;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ConditionOperation;
use solx_mlir::ods::sol::DoWhileOperation;
use solx_mlir::ods::sol::ForOperation;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::WhileOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_for_effect::EmitForEffect;
use crate::ast::emit::emit_statement::EmitStatement;

/// Terminates a Sol op region with `sol.yield`: at `end` if the body fell through to it, otherwise
/// in a fresh dead block, since a body that diverged (return/break/continue) leaves the region with
/// no fall-through block to carry the terminator.
trait YieldTerminator<'context, 'block> {
    fn terminate_with_yield(
        &self,
        context: &StatementContext<'_, 'context, 'block>,
        end: Option<BlockRef<'context, 'block>>,
    );
}

impl<'context, 'block> YieldTerminator<'context, 'block> for RegionRef<'context, '_> {
    fn terminate_with_yield(
        &self,
        context: &StatementContext<'_, 'context, 'block>,
        end: Option<BlockRef<'context, 'block>>,
    ) {
        match end {
            Some(end) => {
                mlir_op_void!(context.state, &end, YieldOperation.ins(&[]));
            }
            None => {
                let dead_block = Block::new(&[]);
                mlir_op_void!(context.state, &dead_block, YieldOperation.ins(&[]));
                self.append_block(dead_block);
            }
        }
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
        .is_nonzero(context.state, &block)
        .into_mlir();

    let (then_block, else_block) = mlir_region_op!(
        context.state, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let then_region = then_block.parent_region().expect("block belongs to a region");
    let else_region = else_block.parent_region().expect("block belongs to a region");

    let then_end = node.body().emit(context, then_block);
    then_region.terminate_with_yield(context, then_end);

    if let Some(ref else_statement) = node.else_branch() {
        let else_end = else_statement.emit(context, else_block);
        else_region.terminate_with_yield(context, else_end);
    } else {
        else_region.terminate_with_yield(context, Some(else_block));
    }

    Some(block)
});

statement_emit!(ForStatement; |node, context, block| {
    context.environment.enter_scope();

    let block = match node.initialization() {
        ForStatementInitialization::VariableDeclarationStatement(declaration) => declaration
            .emit(context, block)
            .expect("a variable declaration statement does not diverge"),
        ForStatementInitialization::ExpressionStatement(expression_statement) => {
            expression_statement
                .emit(context, block)
                .expect("an expression statement does not diverge")
        }
        ForStatementInitialization::Semicolon(_) => block,
    };

    let (condition_block, body_block, step_block) =
        mlir_region_op!(context.state, &block, ForOperation; cond, body, step);
    let body_region = body_block.parent_region().expect("block belongs to a region");

    match node.condition() {
        ForStatementCondition::ExpressionStatement(expression_statement) => {
            let emitter = ExpressionContext::from(&*context);
            let BlockAnd {
                value: condition_value,
                block: condition_end,
            } = expression_statement.expression().emit(&emitter, condition_block);
            let condition_boolean = condition_value
                .is_nonzero(context.state, &condition_end)
                .into_mlir();
            mlir_op_void!(
                context.state,
                &condition_end,
                ConditionOperation.condition(condition_boolean)
            );
        }
        ForStatementCondition::Semicolon(_) => {
            let true_value =
                AstValue::boolean(true, context.state, &condition_block)
                    .into_mlir();
            mlir_op_void!(
                context.state,
                &condition_block,
                ConditionOperation.condition(true_value)
            );
        }
    }

    let body_end = node.body().emit(context, body_block);
    body_region.terminate_with_yield(context, body_end);

    if let Some(ref iterator_expression) = node.iterator() {
        let emitter = ExpressionContext::new(
            context.state,
            context.environment,
            context.dispatch,
            context.storage_layout,
            ArithmeticMode::Unchecked,
        );
        let step_end = iterator_expression.emit_for_effect(&emitter, step_block);
        mlir_op_void!(context.state, &step_end, YieldOperation.ins(&[]));
    } else {
        mlir_op_void!(context.state, &step_block, YieldOperation.ins(&[]));
    }

    context.environment.exit_scope();
    Some(block)
});

statement_emit!(WhileStatement; |node, context, block| {
    let (condition_block, body_block) =
        mlir_region_op!(context.state, &block, WhileOperation; cond, body);
    let body_region = body_block.parent_region().expect("block belongs to a region");

    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: condition_value,
        block: condition_end,
    } = node.condition().emit(&emitter, condition_block);
    let condition_boolean = condition_value
        .is_nonzero(context.state, &condition_end)
        .into_mlir();
    mlir_op_void!(
        context.state,
        &condition_end,
        ConditionOperation.condition(condition_boolean)
    );

    let body_end = node.body().emit(context, body_block);
    body_region.terminate_with_yield(context, body_end);

    Some(block)
});

statement_emit!(DoWhileStatement; |node, context, block| {
    let (body_block, condition_block) =
        mlir_region_op!(context.state, &block, DoWhileOperation; body, cond);
    let body_region = body_block.parent_region().expect("block belongs to a region");

    let body_end = node.body().emit(context, body_block);
    body_region.terminate_with_yield(context, body_end);

    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: condition_value,
        block: condition_end,
    } = node.condition().emit(&emitter, condition_block);
    let condition_boolean = condition_value
        .is_nonzero(context.state, &condition_end)
        .into_mlir();
    mlir_op_void!(
        context.state,
        &condition_end,
        ConditionOperation.condition(condition_boolean)
    );

    Some(block)
});
