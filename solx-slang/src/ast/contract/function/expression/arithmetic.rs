//!
//! Binary arithmetic expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::AdditiveExpressionOperator;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::MultiplicativeExpressionOperator;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PostfixExpressionOperator;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::PrefixExpressionOperator;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

/// A binary arithmetic operation, abstracting over the source operator so the
/// checked/unchecked Sol op selection lives in one place.
#[derive(Debug, Clone, Copy)]
enum ArithmeticOperation {
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Remainder,
    /// `**`
    Exponentiation,
}

impl ArithmeticOperation {
    /// Emits this operator's Sol op through the builder and returns its result.
    ///
    /// In checked mode (Solidity 0.8+ default) the overflow-trapping variants
    /// are emitted; inside `unchecked {}` the wrapping variants are. `%` has no
    /// checked variant.
    fn emit<'context, 'block>(
        self,
        checked: bool,
        builder: &Builder<'context>,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match (self, checked) {
            (Self::Add, true) => builder.emit_sol_cadd(lhs, rhs, block),
            (Self::Add, false) => builder.emit_sol_add(lhs, rhs, block),
            (Self::Subtract, true) => builder.emit_sol_csub(lhs, rhs, block),
            (Self::Subtract, false) => builder.emit_sol_sub(lhs, rhs, block),
            (Self::Multiply, true) => builder.emit_sol_cmul(lhs, rhs, block),
            (Self::Multiply, false) => builder.emit_sol_mul(lhs, rhs, block),
            (Self::Divide, true) => builder.emit_sol_cdiv(lhs, rhs, block),
            (Self::Divide, false) => builder.emit_sol_div(lhs, rhs, block),
            (Self::Remainder, _) => builder.emit_sol_mod(lhs, rhs, block),
            (Self::Exponentiation, true) => builder.emit_sol_cexp(lhs, rhs, block),
            (Self::Exponentiation, false) => builder.emit_sol_exp(lhs, rhs, block),
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an additive expression (`+`, `-`).
    pub(super) fn emit_additive(
        &self,
        expression: &AdditiveExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            AdditiveExpressionOperator::Plus(_) => ArithmeticOperation::Add,
            AdditiveExpressionOperator::Minus(_) => ArithmeticOperation::Subtract,
        };
        let result_type = expression
            .get_type()
            .expect("binder types every arithmetic expression");
        self.emit_binary_arithmetic(
            operation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers a multiplicative expression (`*`, `/`, `%`).
    pub(super) fn emit_multiplicative(
        &self,
        expression: &MultiplicativeExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            MultiplicativeExpressionOperator::Asterisk(_) => ArithmeticOperation::Multiply,
            MultiplicativeExpressionOperator::Slash(_) => ArithmeticOperation::Divide,
            MultiplicativeExpressionOperator::Percent(_) => ArithmeticOperation::Remainder,
        };
        let result_type = expression
            .get_type()
            .expect("binder types every arithmetic expression");
        self.emit_binary_arithmetic(
            operation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers an exponentiation expression (`**`).
    pub(super) fn emit_exponentiation(
        &self,
        expression: &ExponentiationExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = expression
            .get_type()
            .expect("binder types every arithmetic expression");
        self.emit_binary_arithmetic(
            ArithmeticOperation::Exponentiation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Emits a binary arithmetic operation.
    ///
    /// Both operands are coerced to the expression's binder-assigned type so
    /// the Sol op satisfies `SameOperandsAndResultType` and matches solc's
    /// type-annotated IR.
    fn emit_binary_arithmetic(
        &self,
        operation: ArithmeticOperation,
        left: &Expression,
        right: &Expression,
        result_slang_type: &SlangType,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type =
            TypeConversion::resolve_slang_type(result_slang_type, None, &self.state.builder);
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, result_type, block)?;
        let value = operation.emit(self.checked, &self.state.builder, lhs, rhs, &block);
        Ok((value, block))
    }

    /// Emits both operands of a binary expression — right-to-left, matching
    /// solc's evaluation order — and coerces each to `result_type` so the Sol
    /// op satisfies `SameOperandsAndResultType`. Shared with the bitwise domain.
    pub(super) fn emit_binary_operands(
        &self,
        left: &Expression,
        right: &Expression,
        result_type: Type<'context>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (rhs, block) = self.emit_value(right, block)?;
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        Ok((lhs, rhs, block))
    }

    /// Lowers a postfix step (`x++`, `x--`), yielding the value before the step.
    pub(super) fn emit_postfix(
        &self,
        expression: &PostfixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            PostfixExpressionOperator::PlusPlus(_) => ArithmeticOperation::Add,
            PostfixExpressionOperator::MinusMinus(_) => ArithmeticOperation::Subtract,
        };
        let (old, _new, block) =
            self.emit_increment_decrement(operation, &expression.operand(), block)?;
        Ok((old, block))
    }

    /// Lowers a prefix operator, routing each to its domain: `++`/`--` step
    /// here, `!` to logical, `~` to bitwise, `-` to negation. `delete` defers.
    pub(super) fn emit_prefix(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let step = match expression.operator() {
            PrefixExpressionOperator::PlusPlus(_) => ArithmeticOperation::Add,
            PrefixExpressionOperator::MinusMinus(_) => ArithmeticOperation::Subtract,
            PrefixExpressionOperator::Bang(_) => {
                return self.emit_not(&expression.operand(), block);
            }
            PrefixExpressionOperator::Tilde(_) => return self.emit_bitwise_not(expression, block),
            PrefixExpressionOperator::Minus(_) => return self.emit_negate(expression, block),
            PrefixExpressionOperator::DeleteKeyword(_) => {
                unimplemented!("delete operator lowering")
            }
        };
        let (_old, new, block) =
            self.emit_increment_decrement(step, &expression.operand(), block)?;
        Ok((new, block))
    }

    /// Lowers unary negation `-x` as `0 - x` (unchecked subtraction; checked
    /// negation would need signed-type-aware handling of `-INT_MIN`).
    fn emit_negate(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = TypeConversion::resolve_slang_type(
            &expression
                .get_type()
                .expect("binder types every negation expression"),
            None,
            &self.state.builder,
        );
        let (value, block) = self.emit_value(&expression.operand(), block)?;
        let value = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        let zero = self.state.builder.emit_sol_constant(0, result_type, &block);
        let result = self.state.builder.emit_sol_sub(zero, value, &block);
        Ok((result, block))
    }

    /// Emits a `±1` read-modify-write of an lvalue, returning both the value
    /// before the step and the value after it.
    fn emit_increment_decrement(
        &self,
        operation: ArithmeticOperation,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let lvalue = self.resolve_lvalue(operand);
        let element_type = lvalue.element_type();
        let old = self.emit_lvalue_load(&lvalue, &block)?;
        let one = self
            .state
            .builder
            .emit_sol_constant(1, element_type, &block);
        let new = operation.emit(self.checked, &self.state.builder, old, one, &block);
        self.emit_lvalue_store(&lvalue, new, &block);
        Ok((old, new, block))
    }
}
