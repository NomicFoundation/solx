//!
//! Comparison expression lowering to `sol.cmp`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::EqualityExpressionOperator;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;
use slang_solidity_v2::ast::InequalityExpressionOperator;

use solx_mlir::CmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an equality expression (`==`, `!=`).
    pub fn emit_equality(
        &self,
        expression: &EqualityExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let predicate = match expression.operator() {
            EqualityExpressionOperator::EqualEqual(_) => CmpPredicate::Eq,
            EqualityExpressionOperator::BangEqual(_) => CmpPredicate::Ne,
        };
        self.emit_comparison(
            &expression.left_operand(),
            &expression.right_operand(),
            predicate,
            block,
        )
    }

    /// Lowers an inequality expression (`<`, `<=`, `>`, `>=`).
    pub fn emit_inequality(
        &self,
        expression: &InequalityExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let predicate = match expression.operator() {
            InequalityExpressionOperator::LessThan(_) => CmpPredicate::Lt,
            InequalityExpressionOperator::LessThanEqual(_) => CmpPredicate::Le,
            InequalityExpressionOperator::GreaterThan(_) => CmpPredicate::Gt,
            InequalityExpressionOperator::GreaterThanEqual(_) => CmpPredicate::Ge,
        };
        self.emit_comparison(
            &expression.left_operand(),
            &expression.right_operand(),
            predicate,
            block,
        )
    }

    /// Emits a `sol.cmp` over two operands, yielding an `i1`.
    ///
    /// Both operands are coerced to a common type before comparison: their
    /// shared type when equal, otherwise `ui256`. Operands are evaluated
    /// left-to-right.
    fn emit_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        predicate: CmpPredicate,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let (rhs, block) = self.emit_value(right, block)?;

        let common_type = if lhs.r#type() == rhs.r#type() {
            lhs.r#type()
        } else {
            self.state.builder.types.ui256
        };
        let lhs = TypeConversion::from_target_type(common_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(common_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );

        let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
        Ok((comparison, block))
    }
}
