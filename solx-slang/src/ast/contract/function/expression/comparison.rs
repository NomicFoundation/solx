//!
//! Comparison expression emission: equality and inequality (`sol.cmp`), reconciling the operand types first.
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
    // A user-defined comparison operator (`using {f as ==} for T global;`) dispatches to the bound
    // function instead of emitting native `sol.cmp`, mirroring the arithmetic operator bindings in
    // `Operator::emit_binary`. The binding is keyed on the left operand's user-defined value type.
    if let Some(function_id) =
        Operator::user_defined_operator(context, &left, predicate.user_defined_operator())
    {
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        let result = Operator::emit_operator_call(context, function_id, vec![lhs, rhs], &block);
        return BlockAnd { block, value: result.into() };
    }
    // A string literal compared with a `bytesN` sibling materialises toward the sibling's fixed-bytes
    // type (the non-string operand is emitted first to learn it).
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
        let comparison = lhs.compare(rhs, predicate, &context.state.builder, &block);
        return BlockAnd { block, value: comparison };
    }
    // Two fixed-bytes operands of different widths: `bytesN` are LEFT-aligned, so widen the smaller
    // and compare AS fixed-bytes (matching solc). The mixed-integer path below would right-align them,
    // yielding the wrong result.
    if let (Some(lhs_width), Some(rhs_width)) = (
        lhs.r#type().fixed_bytes_or_byte_width(),
        rhs.r#type().fixed_bytes_or_byte_width(),
    ) {
        let builder = &context.state.builder;
        let common_width = lhs_width.max(rhs_width);
        let common = AstType::fixed_bytes(builder.context, common_width).into_mlir();
        let lhs_common = if lhs_width == common_width {
            lhs
        } else {
            lhs.cast(AstType::new(common), builder, &block)
        };
        let rhs_common = if rhs_width == common_width {
            rhs
        } else {
            rhs.cast(AstType::new(common), builder, &block)
        };
        let comparison = lhs_common.compare(rhs_common, predicate, builder, &block);
        return BlockAnd { block, value: comparison };
    }
    // Mixed-type comparison: widen each operand to 256 bits preserving ITS OWN signedness (so a
    // negative isn't reinterpreted as a huge unsigned), then compare as signed if either is signed
    // — a plain `ui256` default would make `(-10) < 10` an unsigned (false) comparison.
    let signed_lhs =
        IntegerType::try_from(lhs.r#type().into_mlir()).is_ok_and(|integer| integer.is_signed());
    let signed_rhs =
        IntegerType::try_from(rhs.r#type().into_mlir()).is_ok_and(|integer| integer.is_signed());
    let mlir_context = context.state.builder.context;
    let signed_256 = Type::from(IntegerType::signed(mlir_context, 256));
    let unsigned_256 =
        AstType::unsigned(mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir();
    let lhs_wide_type = if signed_lhs { signed_256 } else { unsigned_256 };
    let rhs_wide_type = if signed_rhs { signed_256 } else { unsigned_256 };
    let lhs_wide = lhs.cast(
        AstType::new(lhs_wide_type),
        &context.state.builder,
        &block,
    );
    let rhs_wide = rhs.cast(
        AstType::new(rhs_wide_type),
        &context.state.builder,
        &block,
    );
    let common = if signed_lhs || signed_rhs {
        signed_256
    } else {
        unsigned_256
    };
    let lhs_common = if lhs_wide.r#type().into_mlir() == common {
        lhs_wide
    } else {
        lhs_wide.cast(AstType::new(common), &context.state.builder, &block)
    };
    let rhs_common = if rhs_wide.r#type().into_mlir() == common {
        rhs_wide
    } else {
        rhs_wide.cast(AstType::new(common), &context.state.builder, &block)
    };
    let comparison = lhs_common.compare(rhs_common, predicate, &context.state.builder, &block);
    BlockAnd { block, value: comparison }
});
