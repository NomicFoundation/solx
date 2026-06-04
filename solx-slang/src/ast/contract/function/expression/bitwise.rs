//!
//! Bitwise and shift expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::ShiftExpression;
use slang_solidity_v2::ast::ShiftExpressionOperator;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// A bitwise or shift binary operation. Unlike arithmetic, these have no
/// overflow-checked variant.
#[derive(Debug, Clone, Copy)]
pub enum BitwiseOperation {
    /// `&`
    And,
    /// `|`
    Or,
    /// `^`
    Xor,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
}

impl BitwiseOperation {
    /// Emits this operator's Sol op through the builder and returns its result.
    pub fn emit<'context, 'block>(
        self,
        builder: &Builder<'context>,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match self {
            Self::And => builder.emit_sol_and(lhs, rhs, block),
            Self::Or => builder.emit_sol_or(lhs, rhs, block),
            Self::Xor => builder.emit_sol_xor(lhs, rhs, block),
            Self::ShiftLeft => builder.emit_sol_shl(lhs, rhs, block),
            Self::ShiftRight => builder.emit_sol_shr(lhs, rhs, block),
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a bitwise-and expression (`&`).
    pub fn emit_bitwise_and(
        &self,
        expression: &BitwiseAndExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = expression
            .get_type()
            .expect("binder types every bitwise expression");
        self.emit_binary_bitwise(
            BitwiseOperation::And,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers a bitwise-or expression (`|`).
    pub fn emit_bitwise_or(
        &self,
        expression: &BitwiseOrExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = expression
            .get_type()
            .expect("binder types every bitwise expression");
        self.emit_binary_bitwise(
            BitwiseOperation::Or,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers a bitwise-xor expression (`^`).
    pub fn emit_bitwise_xor(
        &self,
        expression: &BitwiseXorExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = expression
            .get_type()
            .expect("binder types every bitwise expression");
        self.emit_binary_bitwise(
            BitwiseOperation::Xor,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers a shift expression (`<<`, `>>`, `>>>`).
    pub fn emit_shift(
        &self,
        expression: &ShiftExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            ShiftExpressionOperator::LessThanLessThan(_) => BitwiseOperation::ShiftLeft,
            ShiftExpressionOperator::GreaterThanGreaterThan(_)
            | ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => {
                BitwiseOperation::ShiftRight
            }
        };
        let result_type = expression
            .get_type()
            .expect("binder types every shift expression");
        self.emit_binary_bitwise(
            operation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers a bitwise-not expression (`~x`) to `sol.not`.
    pub fn emit_bitwise_not(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = TypeConversion::resolve_slang_type(
            &expression
                .get_type()
                .expect("binder types every bitwise-not expression"),
            None,
            &self.state.builder,
        );
        let (value, block) = self.emit_value(&expression.operand(), block)?;
        let value = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        let result = self.state.builder.emit_sol_not(value, &block);
        Ok((result, block))
    }

    /// Emits a bitwise/shift operation: both operands are coerced to the
    /// expression's binder-assigned type, then the Sol op is emitted.
    fn emit_binary_bitwise(
        &self,
        operation: BitwiseOperation,
        left: &Expression,
        right: &Expression,
        result_slang_type: &SlangType,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type =
            TypeConversion::resolve_slang_type(result_slang_type, None, &self.state.builder);
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, result_type, block)?;
        let value = operation.emit(&self.state.builder, lhs, rhs, &block);
        Ok((value, block))
    }
}
