//!
//! Conditional (ternary) expression emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use slang_solidity_v2::ast::ConditionalExpression;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(ConditionalExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type()).unwrap_or_else(|| {
        AstType::unsigned(context.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
    });
    let condition = node.operand();
    let BlockAnd {
        value: condition_value,
        block,
    } = condition.emit(context, block);
    let condition_boolean = context.emit_is_nonzero(condition_value, &block);

    let result_slot = Pointer::stack(AstType::new(result_type), context.state, &block);
    let (then_block, else_block) = mlir_region_op!(
        context.state, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let true_expression = node.true_expression();
    let BlockAnd {
        value: then_value,
        block: then_end,
    } = true_expression.emit(context, then_block);
    let then_cast = TypeConversion::from_target_type(result_type, context.state)
        .emit(then_value, context.state, &then_end);
    result_slot.store(AstValue::new(then_cast), context.state, &then_end);
    mlir_op_void!(context.state, &then_end, YieldOperation.ins(&[]));

    let false_expression = node.false_expression();
    let BlockAnd {
        value: else_value,
        block: else_end,
    } = false_expression.emit(context, else_block);
    let else_cast = TypeConversion::from_target_type(result_type, context.state)
        .emit(else_value, context.state, &else_end);
    result_slot.store(AstValue::new(else_cast), context.state, &else_end);
    mlir_op_void!(context.state, &else_end, YieldOperation.ins(&[]));

    let value = result_slot
        .load(AstType::new(result_type), context.state, &block)
        .into_mlir();
    BlockAnd { block, value }
});
