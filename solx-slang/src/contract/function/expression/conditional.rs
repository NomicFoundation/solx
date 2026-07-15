//!
//! The ternary conditional operator.
//!

use slang_solidity_v2::ast::ConditionalExpression;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The ternary conditional operator: neither operand short-circuits, so there is no initializer
    /// and both arms store their evaluated operand.
    pub fn conditional(&mut self, node: &ConditionalExpression) -> Value<'context> {
        let result_type = self.typing(node.get_type());
        let condition = self.expression(&node.operand()).is_nonzero(self);
        self.branch_value(
            condition,
            result_type,
            |_scope| None,
            |scope| Some(scope.expression(&node.true_expression())),
            |scope| Some(scope.expression(&node.false_expression())),
        )
    }
}
