//!
//! Bitwise and shift expression lowering.
//!

use melior::Context as MlirContext;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Value;
use melior::ir::operation::Operation;
use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::ShiftExpression;
use slang_solidity_v2::ast::ShiftExpressionOperator;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::AndOperation;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::OrOperation;
use solx_mlir::ods::sol::ShlOperation;
use solx_mlir::ods::sol::ShrOperation;
use solx_mlir::ods::sol::XorOperation;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

/// A bitwise or shift binary operation. Unlike arithmetic, these have no
/// overflow-checked variant.
#[derive(Debug, Clone, Copy)]
enum BitwiseOperation {
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
    /// Builds the Sol dialect operation for this operator. The result type is
    /// inferred from the operands (`SameOperandsAndResultType`).
    fn build<'context, 'block>(
        self,
        context: &'context MlirContext,
        location: Location<'context>,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
    ) -> Operation<'context> {
        match self {
            Self::And => AndOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Or => OrOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Xor => XorOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftLeft => ShlOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftRight => ShrOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a bitwise-and expression (`&`).
    pub(super) fn emit_bitwise_and(
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
    pub(super) fn emit_bitwise_or(
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
    pub(super) fn emit_bitwise_xor(
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
    pub(super) fn emit_shift(
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
    pub(super) fn emit_bitwise_not(
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
        let operation = NotOperation::builder(
            self.state.builder.context,
            self.state.builder.unknown_location,
        )
        .value(value)
        .build();
        let result = block
            .append_operation(operation.into())
            .result(0)
            .expect("sol.not produces one result")
            .into();
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
        let operation = operation.build(
            self.state.builder.context,
            self.state.builder.unknown_location,
            lhs,
            rhs,
        );
        let value = block
            .append_operation(operation)
            .result(0)
            .expect("bitwise operation produces one result")
            .into();
        Ok((value, block))
    }
}
