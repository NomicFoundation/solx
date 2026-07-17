//!
//! Comparison expressions: equality and inequality over reconciled operand types.
//!

use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::EqualityExpressionOperator;
use slang_solidity_v2::ast::InequalityExpression;
use slang_solidity_v2::ast::InequalityExpressionOperator;

use solx_mlir::CmpPredicate;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `a == b` and `a != b`, comparing the operands under the reconciled predicate.
    pub fn equality(&mut self, node: &EqualityExpression) -> Value<'context> {
        let predicate = match node.operator() {
            EqualityExpressionOperator::EqualEqual(_) => CmpPredicate::Eq,
            EqualityExpressionOperator::BangEqual(_) => CmpPredicate::Ne,
        };
        self.expression(&node.left_operand()).compare_coerced(
            self.expression(&node.right_operand()),
            predicate,
            self,
        )
    }

    /// `a < b`, `a <= b`, `a > b`, and `a >= b`, comparing the operands under the reconciled
    /// predicate.
    pub fn inequality(&mut self, node: &InequalityExpression) -> Value<'context> {
        let predicate = match node.operator() {
            InequalityExpressionOperator::LessThan(_) => CmpPredicate::Lt,
            InequalityExpressionOperator::LessThanEqual(_) => CmpPredicate::Le,
            InequalityExpressionOperator::GreaterThan(_) => CmpPredicate::Gt,
            InequalityExpressionOperator::GreaterThanEqual(_) => CmpPredicate::Ge,
        };
        self.expression(&node.left_operand()).compare_coerced(
            self.expression(&node.right_operand()),
            predicate,
            self,
        )
    }
}
