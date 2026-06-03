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
use solx_mlir::TypeFactory;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an equality expression (`==`, `!=`).
    pub(super) fn emit_equality(
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
    pub(super) fn emit_inequality(
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

    /// Coerces a value to an `i1` boolean condition for control flow.
    ///
    /// A value already of width 1 is returned unchanged; any wider integer is
    /// compared against zero (`!= 0`).
    pub(crate) fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        if TypeFactory::integer_bit_width(value.r#type()) == 1 {
            return value;
        }
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, value.r#type(), block);
        self.state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Ne, block)
    }
}
