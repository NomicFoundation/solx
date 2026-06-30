//!
//! Comparison expression emission: equality and inequality, reconciling the operand types first.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;
use solx_mlir::CmpPredicate;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::operator::Operator;

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
    if lhs.r#type() == rhs.r#type() {
        let comparison = lhs.compare(rhs, predicate, context.state, &block);
        return BlockAnd { block, value: comparison };
    }
    let lhs_bytes = lhs.r#type().fixed_bytes_or_byte_width();
    let rhs_bytes = rhs.r#type().fixed_bytes_or_byte_width();
    if let Some(common_width) = lhs_bytes.into_iter().chain(rhs_bytes).max() {
        let state = context.state;
        let common = AstType::fixed_bytes(state.mlir_context, common_width).into_mlir();
        let lhs_common = lhs.cast(AstType::new(common), state, &block);
        let rhs_common = rhs.cast(AstType::new(common), state, &block);
        let comparison = lhs_common.compare(rhs_common, predicate, state, &block);
        return BlockAnd { block, value: comparison };
    }
    // Widen each operand to 256 bits preserving its OWN signedness (so a negative is not
    // reinterpreted as a huge unsigned), then compare as signed if either is signed. A plain
    // ui256 default would make `(-10) < 10` a false unsigned comparison.
    let signed_lhs = lhs.r#type().is_signed();
    let signed_rhs = rhs.r#type().is_signed();
    let mlir_context = context.state.mlir_context;
    let signed_256 = Type::from(IntegerType::signed(mlir_context, 256));
    let unsigned_256 =
        AstType::unsigned(mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir();
    let lhs_wide_type = if signed_lhs { signed_256 } else { unsigned_256 };
    let rhs_wide_type = if signed_rhs { signed_256 } else { unsigned_256 };
    let lhs_wide = lhs.cast(
        AstType::new(lhs_wide_type),
        context.state,
        &block,
    );
    let rhs_wide = rhs.cast(
        AstType::new(rhs_wide_type),
        context.state,
        &block,
    );
    let common = if signed_lhs || signed_rhs {
        signed_256
    } else {
        unsigned_256
    };
    let lhs_common = lhs_wide.cast(AstType::new(common), context.state, &block);
    let rhs_common = rhs_wide.cast(AstType::new(common), context.state, &block);
    let comparison = lhs_common.compare(rhs_common, predicate, context.state, &block);
    BlockAnd { block, value: comparison }
});
