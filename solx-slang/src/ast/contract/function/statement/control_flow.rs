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
use solx_mlir::ods::sol::DoWhileOperation;
use solx_mlir::ods::sol::ForOperation;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::WhileOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::EmitStatement;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::statement::StatementContext;
use melior::ir::Block;

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

    let (then_block, else_block) = mlir_region_op!(
        &context.state.builder, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let then_region = solx_mlir::ffi::block_parent_region(&then_block);
    let else_region = solx_mlir::ffi::block_parent_region(&else_block);

    let saved_region = context.region_pointer;
    context.region_pointer = &*then_region as *const _;
    let then_end = node.body().emit(context, then_block);
    if let Some(then_end) = then_end {
        mlir_op_void!(&context.state.builder, &then_end, YieldOperation.ins(&[]));
    } else {
        let dead_block = Block::new(&[]);
        mlir_op_void!(&context.state.builder, &dead_block, YieldOperation.ins(&[]));
        then_region.append_block(dead_block);
    }

    if let Some(ref else_statement) = node.else_branch() {
        context.region_pointer = &*else_region as *const _;
        let else_end = else_statement.emit(context, else_block);
        if let Some(else_end) = else_end {
            mlir_op_void!(&context.state.builder, &else_end, YieldOperation.ins(&[]));
        } else {
            let dead_block = Block::new(&[]);
            mlir_op_void!(&context.state.builder, &dead_block, YieldOperation.ins(&[]));
            else_region.append_block(dead_block);
        }
        context.region_pointer = saved_region;
    } else {
        mlir_op_void!(&context.state.builder, &else_block, YieldOperation.ins(&[]));
        context.region_pointer = saved_region;
    }

    Some(block)
});

statement_emit!(ForStatement; |node, context, block| {
    context.environment.enter_scope();

    let block = match node.initialization() {
        ForStatementInitialization::VariableDeclarationStatement(declaration) => {
            let statement = Statement::VariableDeclarationStatement(declaration);
            match statement.emit(context, block) {
                Some(block) => block,
                None => {
                    context.environment.exit_scope();
                    return None;
                }
            }
        }
        ForStatementInitialization::ExpressionStatement(expression_statement) => {
            let statement = Statement::ExpressionStatement(expression_statement);
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

    let (condition_block, body_block, step_block) =
        mlir_region_op!(&context.state.builder, &block, ForOperation; cond, body, step);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    match node.condition() {
        ForStatementCondition::ExpressionStatement(expression_statement) => {
            let emitter = ExpressionContext::from(&*context);
            let BlockAnd {
                value: condition_value,
                block: condition_end,
            } = expression_statement.expression().emit(&emitter, condition_block);
            let condition_boolean = condition_value
                .is_nonzero(&context.state.builder, &condition_end)
                .into_mlir();
            mlir_op_void!(
                &context.state.builder,
                &condition_end,
                ConditionOperation.condition(condition_boolean)
            );
        }
        ForStatementCondition::Semicolon(_) => {
            let true_value =
                AstValue::boolean(true, &context.state.builder, &condition_block)
                    .into_mlir();
            mlir_op_void!(
                &context.state.builder,
                &condition_block,
                ConditionOperation.condition(true_value)
            );
        }
    }

    context.region_pointer = &*body_region as *const _;
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        mlir_op_void!(&context.state.builder, &body_end, YieldOperation.ins(&[]));
    }

    if let Some(ref iterator_expression) = node.iterator() {
        let emitter = ExpressionContext::new(
            context.state,
            context.environment,
            context.storage_layout,
            ArithmeticMode::Unchecked,
        );
        let step_end = iterator_expression.emit_for_effect(&emitter, step_block);
        mlir_op_void!(&context.state.builder, &step_end, YieldOperation.ins(&[]));
    } else {
        mlir_op_void!(&context.state.builder, &step_block, YieldOperation.ins(&[]));
    }

    context.region_pointer = saved_region;
    context.environment.exit_scope();
    Some(block)
});

statement_emit!(WhileStatement; |node, context, block| {
    let (condition_block, body_block) =
        mlir_region_op!(&context.state.builder, &block, WhileOperation; cond, body);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: condition_value,
        block: condition_end,
    } = node.condition().emit(&emitter, condition_block);
    let condition_boolean = condition_value
        .is_nonzero(&context.state.builder, &condition_end)
        .into_mlir();
    mlir_op_void!(
        &context.state.builder,
        &condition_end,
        ConditionOperation.condition(condition_boolean)
    );

    context.region_pointer = &*body_region as *const _;
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        mlir_op_void!(&context.state.builder, &body_end, YieldOperation.ins(&[]));
    }

    context.region_pointer = saved_region;
    Some(block)
});

statement_emit!(DoWhileStatement; |node, context, block| {
    let (body_block, condition_block) =
        mlir_region_op!(&context.state.builder, &block, DoWhileOperation; body, cond);
    let body_region = solx_mlir::ffi::block_parent_region(&body_block);
    let saved_region = context.region_pointer;

    context.region_pointer = &*body_region as *const _;
    let body_end = node.body().emit(context, body_block);
    if let Some(body_end) = body_end {
        mlir_op_void!(&context.state.builder, &body_end, YieldOperation.ins(&[]));
    }

    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: condition_value,
        block: condition_end,
    } = node.condition().emit(&emitter, condition_block);
    let condition_boolean = condition_value
        .is_nonzero(&context.state.builder, &condition_end)
        .into_mlir();
    mlir_op_void!(
        &context.state.builder,
        &condition_end,
        ConditionOperation.condition(condition_boolean)
    );

    context.region_pointer = saved_region;
    Some(block)
});
