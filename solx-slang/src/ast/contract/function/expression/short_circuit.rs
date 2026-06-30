//!
//! Short-circuit logical expression emission: `&&` and `||`. Each node bridges
//! to its [`LogicalOperator`], which emits itself.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::AndExpression;
use slang_solidity_v2::ast::OrExpression;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::logical_operator::LogicalOperator;

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
    );
    BlockAnd { block, value: value.into() }
});
