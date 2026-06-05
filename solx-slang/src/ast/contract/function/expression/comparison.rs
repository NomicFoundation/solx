//!
//! Comparison expression lowering to `sol.cmp`.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::EqualityExpressionOperator;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;
use slang_solidity_v2::ast::InequalityExpressionOperator;

use solx_mlir::CmpPredicate;
use solx_mlir::TypeFactory;

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
        // A string literal compared to a `bytesN` operand must be materialized
        // as that fixedbytes value, not the string-memory value `emit_value`
        // produces (which `sol.cast` cannot coerce to the comparison type).
        if let Some(result) =
            self.emit_fixed_bytes_string_comparison(left, right, predicate, block)?
        {
            return Ok(result);
        }

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

    /// Lowers a comparison where one operand is a string literal and the other a
    /// `bytesN`, materializing the literal as that fixedbytes value (left-aligned,
    /// zero-padded) so both operands share the fixedbytes type. Returns `Ok(None)`
    /// when neither operand is such a pairing, leaving the general path untouched.
    fn emit_fixed_bytes_string_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        predicate: CmpPredicate,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Value<'context, 'block>, BlockRef<'context, 'block>)>> {
        // Identify the (string literal, other operand) pairing in either order.
        let (literal, other, literal_on_left) = match (left, right) {
            (Expression::StringExpression(literal), _) => (literal, right, true),
            (_, Expression::StringExpression(literal)) => (literal, left, false),
            _ => return Ok(None),
        };
        let Some(other_slang_type) = other.get_type() else {
            return Ok(None);
        };
        let other_type =
            TypeConversion::resolve_slang_type(&other_slang_type, None, &self.state.builder);
        let Some(width) = TypeFactory::fixed_bytes_width(other_type) else {
            return Ok(None);
        };

        // The literal occupies the high-order bytes, zero-padded on the right.
        let mut buffer = vec![0u8; width as usize];
        for (slot, byte) in buffer.iter_mut().zip(literal.value().iter()) {
            *slot = *byte;
        }
        let integer_type = Type::from(IntegerType::unsigned(self.state.builder.context, width * 8));
        let integer = self.state.builder.emit_constant(
            &BigInt::from_bytes_be(num_bigint::Sign::Plus, &buffer),
            integer_type,
            &block,
        );
        let literal_value = self.state.builder.emit_sol_bytes_cast(
            integer,
            self.state.builder.types.fixed_bytes(width),
            &block,
        );

        let (other_value, block) = self.emit_value(other, block)?;
        let (lhs, rhs) = if literal_on_left {
            (literal_value, other_value)
        } else {
            (other_value, literal_value)
        };
        let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
        Ok(Some((comparison, block)))
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
