//!
//! Comparison expression emission: equality and inequality (`sol.cmp`), reconciling the operand types first.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::InequalityExpression;
use solx_mlir::CmpPredicate;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;

expression_emit!(EqualityExpression, InequalityExpression; |node, context, block| {
    let left = node.left_operand();
    let right = node.right_operand();
    let predicate = CmpPredicate::from(node.operator());
    let BlockAnd { value: lhs, block } = left.emit(context, block);
    let BlockAnd { value: rhs, block } = right.emit(context, block);
    if lhs.r#type() == rhs.r#type() {
        let comparison = lhs.compare(rhs, predicate, &context.state.builder, &block);
        return BlockAnd { block, value: comparison };
    }
    // Mixed-type comparison: cast both operands to `ui256` and compare.
    let common = AstType::unsigned(context.state.builder.context, solx_utils::BIT_LENGTH_FIELD);
    let lhs = lhs.cast(common, &context.state.builder, &block);
    let rhs = rhs.cast(common, &context.state.builder, &block);
    let comparison = lhs.compare(rhs, predicate, &context.state.builder, &block);
    BlockAnd { block, value: comparison }
});
