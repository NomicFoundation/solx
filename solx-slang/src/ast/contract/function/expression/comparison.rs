//!
//! Comparison expression emission: equality and inequality (`sol.cmp`). Each
//! node's `Emit` projects its typed slang operator enum to the [`CmpPredicate`]
//! it applies — via `CmpPredicate::from`, homed on the predicate in solx-mlir —
//! and emits `sol.cmp`, reconciling the operand types first.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;
use solx_mlir::CmpPredicate;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::contract::function::expression::ExpressionContext;

expression_emit!(EqualityExpression, InequalityExpression; |node, context, block| {
    let left = node.left_operand();
    let right = node.right_operand();
    let predicate = CmpPredicate::from(node.operator());
    // A string literal compared with a `bytesN` / `byte` sibling (`b == "d"`)
    // materialises toward the sibling's fixed-bytes type rather than emitting a
    // runtime `sol.string`; the non-string operand is emitted first to learn
    // that type. With neither (or both) a string literal, both emit naturally.
    let left_is_string = matches!(left, Expression::StringExpression(_));
    let right_is_string = matches!(right, Expression::StringExpression(_));
    let (lhs, rhs, block) = if right_is_string && !left_is_string {
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        // `b == "d"`: the string literal materialises toward the non-string
        // sibling's fixed-bytes type, the sibling emitted first to learn it.
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
    // Reconcile the operand types (fixed-bytes width / mixed integer signedness)
    // and emit `sol.cmp`, homed on the value.
    let comparison = lhs.compare_coerced(rhs, predicate, &context.state.builder, &block);
    BlockAnd { block, value: comparison }
});
