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

/// Bridges a slang short-circuit node to the [`LogicalOperator`] it applies.
trait LogicalOperatorExt {
    /// The [`LogicalOperator`] this node applies.
    fn bridged_operator(&self) -> LogicalOperator;
}

impl LogicalOperatorExt for AndExpression {
    fn bridged_operator(&self) -> LogicalOperator {
        LogicalOperator::And
    }
}

impl LogicalOperatorExt for OrExpression {
    fn bridged_operator(&self) -> LogicalOperator {
        LogicalOperator::Or
    }
}

expression_emit!(AndExpression, OrExpression; |node, context, block| {
    let (value, block) = node.bridged_operator().emit(
        context,
        &node.left_operand(),
        &node.right_operand(),
        block,
    )?;
    Ok(BlockAnd { block, value: value.into() })
});
