//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;
use slang_solidity_v2::ast::Expression;
use solx_mlir::Builder;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::arithmetic::ArithmeticOperation;
use crate::ast::contract::function::expression::bitwise::BitwiseOperation;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::lvalue::Lvalue;

/// The binary operation a compound assignment `x op= y` applies as
/// `x = x op y`.
enum CompoundOperation {
    /// An overflow-checkable arithmetic op (`+=`, `-=`, `*=`, `/=`, `%=`).
    Arithmetic(ArithmeticOperation),
    /// A bitwise or shift op (`&=`, `|=`, `^=`, `<<=`, `>>=`).
    Bitwise(BitwiseOperation),
}

impl CompoundOperation {
    /// The operation a compound-assignment operator applies, or `None` for a
    /// plain `=`.
    fn from_operator(operator: &AssignmentExpressionOperator) -> Option<Self> {
        let operation = match operator {
            AssignmentExpressionOperator::Equal(_) => return None,
            AssignmentExpressionOperator::PlusEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Add)
            }
            AssignmentExpressionOperator::MinusEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Subtract)
            }
            AssignmentExpressionOperator::AsteriskEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Multiply)
            }
            AssignmentExpressionOperator::SlashEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Divide)
            }
            AssignmentExpressionOperator::PercentEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Remainder)
            }
            AssignmentExpressionOperator::AmpersandEqual(_) => Self::Bitwise(BitwiseOperation::And),
            AssignmentExpressionOperator::BarEqual(_) => Self::Bitwise(BitwiseOperation::Or),
            AssignmentExpressionOperator::CaretEqual(_) => Self::Bitwise(BitwiseOperation::Xor),
            AssignmentExpressionOperator::LessThanLessThanEqual(_) => {
                Self::Bitwise(BitwiseOperation::ShiftLeft)
            }
            AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_)
            | AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
                Self::Bitwise(BitwiseOperation::ShiftRight)
            }
        };
        Some(operation)
    }

    /// Applies the operation to `(left, right)` through the builder.
    fn emit<'context, 'block>(
        self,
        checked: bool,
        builder: &Builder<'context>,
        left: Value<'context, 'block>,
        right: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match self {
            Self::Arithmetic(operation) => operation.emit(checked, builder, left, right, block),
            Self::Bitwise(operation) => operation.emit(builder, left, right, block),
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an assignment `x = y` or a compound assignment `x op= y`.
    ///
    /// The stored value — the coerced right-hand side, or `x op y` for a
    /// compound operator — is both written to the target and returned as the
    /// expression's result.
    pub fn emit_assignment(
        &self,
        assignment: &AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lvalue, block) = self.resolve_lvalue(&assignment.left_operand(), block)?;
        let element_type = lvalue.element_type();
        let (value, block) = match CompoundOperation::from_operator(&assignment.operator()) {
            None => self.emit_value(&assignment.right_operand(), block)?,
            Some(operation) => {
                self.emit_compound_value(&lvalue, operation, &assignment.right_operand(), block)?
            }
        };
        let builder = &self.state.builder;
        let stored =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        self.emit_lvalue_store(&lvalue, stored, &block);
        Ok((stored, block))
    }

    /// Computes `x op y` for a compound assignment: loads the target's current
    /// value, evaluates the right-hand side coerced to the target type, and
    /// applies the operation.
    fn emit_compound_value(
        &self,
        lvalue: &Lvalue<'context, 'block>,
        operation: CompoundOperation,
        right_operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let element_type = lvalue.element_type();
        let old = self.emit_lvalue_load(lvalue, &block)?;
        let (right, block) = self.emit_value(right_operand, block)?;
        let builder = &self.state.builder;
        let right =
            TypeConversion::from_target_type(element_type, builder).emit(right, builder, &block);
        let value = operation.emit(self.checked, builder, old, right, &block);
        Ok((value, block))
    }
}
