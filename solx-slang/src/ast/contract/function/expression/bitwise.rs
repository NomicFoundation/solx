//!
//! Bitwise and shift expression lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Value;
use melior::ir::operation::Operation;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ShiftExpression;

use solx_mlir::ods::sol::AndOperation;
use solx_mlir::ods::sol::OrOperation;
use solx_mlir::ods::sol::ShlOperation;
use solx_mlir::ods::sol::ShrOperation;
use solx_mlir::ods::sol::XorOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;

/// A binary bitwise or shift operator. All variants are unchecked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// Builds the Sol dialect operation for this operator. The result type is
    /// inferred from `lhs` (`SameOperandsAndResultType`).
    pub fn emit_operation<'context>(
        self,
        context: &'context melior::Context,
        location: Location<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
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
    pub fn emit_bitwise_and(
        &self,
        expression: &BitwiseAndExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_bitwise(
            &expression.left_operand(),
            &expression.right_operand(),
            BitwiseOperation::And,
            self.resolve_slang_type(expression.get_type()),
            block,
        )
    }

    /// Lowers a bitwise-or expression (`|`).
    pub fn emit_bitwise_or(
        &self,
        expression: &BitwiseOrExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_bitwise(
            &expression.left_operand(),
            &expression.right_operand(),
            BitwiseOperation::Or,
            self.resolve_slang_type(expression.get_type()),
            block,
        )
    }

    /// Lowers a bitwise-xor expression (`^`).
    pub fn emit_bitwise_xor(
        &self,
        expression: &BitwiseXorExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_bitwise(
            &expression.left_operand(),
            &expression.right_operand(),
            BitwiseOperation::Xor,
            self.resolve_slang_type(expression.get_type()),
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
            ast::ShiftExpressionOperator::LessThanLessThan(_) => BitwiseOperation::ShiftLeft,
            ast::ShiftExpressionOperator::GreaterThanGreaterThan(_)
            | ast::ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => {
                BitwiseOperation::ShiftRight
            }
        };
        self.emit_bitwise(
            &expression.left_operand(),
            &expression.right_operand(),
            operation,
            self.resolve_slang_type(expression.get_type()),
            block,
        )
    }

    /// Emits a binary bitwise operation, typed by the binder's result type.
    fn emit_bitwise(
        &self,
        left: &Expression,
        right: &Expression,
        operation: BitwiseOperation,
        target_type: Option<melior::ir::Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, target_type, block)?;
        let value = block
            .append_operation(operation.emit_operation(
                self.state.builder.context,
                self.state.builder.unknown_location,
                lhs,
                rhs,
            ))
            .result(0)
            .expect("a binary operation always produces one result")
            .into();
        Ok((value, block))
    }
}
