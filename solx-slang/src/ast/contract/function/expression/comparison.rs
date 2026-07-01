//!
//! Comparison expression emission: equality and inequality, reconciling the operand types first.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;

use solx_mlir::CmpPredicate;
use solx_mlir::Type as AstType;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(EqualityExpression, InequalityExpression; |node, context, block| {
    let left = node.left_operand();
    let right = node.right_operand();
    let predicate = CmpPredicate::from(node.operator());
    if let Some(function_id) =
        Operator::user_defined_operator(context, &left, predicate.into())
    {
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        let result = Operator::emit_operator_call(context, function_id, vec![lhs, rhs], &block);
        return BlockAnd { block, value: result.into() };
    }
    let left_is_string = matches!(left, Expression::StringExpression(_));
    let right_is_string = matches!(right, Expression::StringExpression(_));
    let (lhs, rhs, block) = if right_is_string && !left_is_string {
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        let BlockAnd { value: rhs, block } =
            right.emit_as(lhs.r#type().into_mlir(), context, block);
        (lhs, rhs, block)
    } else if left_is_string && !right_is_string {
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        let BlockAnd { value: lhs, block } =
            left.emit_as(rhs.r#type().into_mlir(), context, block);
        (lhs, rhs, block)
    } else {
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        (lhs, rhs, block)
    };
    let state = context.state;
    let lhs_type = lhs.r#type();
    let rhs_type = rhs.r#type();
    let common = if lhs_type == rhs_type {
        lhs_type
    } else if let Some(width) = lhs_type
        .fixed_bytes_or_byte_width()
        .into_iter()
        .chain(rhs_type.fixed_bytes_or_byte_width())
        .max()
    {
        AstType::fixed_bytes(state.mlir_context, width)
    } else if lhs_type.integer_bit_width() >= rhs_type.integer_bit_width() {
        lhs_type
    } else {
        rhs_type
    };
    let comparison = lhs
        .cast(common, state, &block)
        .compare(rhs.cast(common, state, &block), predicate, state, &block);
    BlockAnd { block, value: comparison }
});
