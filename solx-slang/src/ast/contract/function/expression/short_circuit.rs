//!
//! Short-circuit logical expression lowering: `&&` and `||`. Each node bridges
//! to its [`LogicalOperator`], which lowers itself.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::AndExpression;
use slang_solidity_v2::ast::OrExpression;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::logical_operator::LogicalOperator;

// Each slang short-circuit node projects to the [`LogicalOperator`] it applies,
// homed on `LogicalOperator` (a slang-local enum) via `From`, the conversion's
// concept, rather than a bespoke extension trait.

impl From<&AndExpression> for LogicalOperator {
    fn from(_node: &AndExpression) -> Self {
        Self::And
    }
}

impl From<&OrExpression> for LogicalOperator {
    fn from(_node: &OrExpression) -> Self {
        Self::Or
    }
}

expression_emit!(AndExpression, OrExpression; |node, context, block| {
    let (value, block) = LogicalOperator::from(node).emit(
        context,
        &node.left_operand(),
        &node.right_operand(),
        block,
    )?;
    Ok(BlockAnd { block, value: value.into() })
});
