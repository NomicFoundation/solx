//!
//! Binary arithmetic expression lowering.
//!

use melior::Context as MlirContext;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;
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

use solx_mlir::ods::sol::AddOperation;
use solx_mlir::ods::sol::CAddOperation;
use solx_mlir::ods::sol::CDivOperation;
use solx_mlir::ods::sol::CExpOperation;
use solx_mlir::ods::sol::CMulOperation;
use solx_mlir::ods::sol::CSubOperation;
use solx_mlir::ods::sol::DivOperation;
use solx_mlir::ods::sol::ExpOperation;
use solx_mlir::ods::sol::ModOperation;
use solx_mlir::ods::sol::MulOperation;
use solx_mlir::ods::sol::SubOperation;

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
    /// Builds the Sol dialect operation for this arithmetic operator.
    ///
    /// In checked mode (Solidity 0.8+ default) the overflow-trapping variants
    /// `sol.cadd`/`sol.csub`/`sol.cmul`/`sol.cdiv`/`sol.cexp` are emitted;
    /// inside `unchecked {}` the wrapping variants are emitted instead. `%` has
    /// no checked variant. The result type is inferred from the operands
    /// (`SameOperandsAndResultType`), except `**`, whose result is set
    /// explicitly.
    fn build<'context, 'block>(
        self,
        checked: bool,
        context: &'context MlirContext,
        location: Location<'context>,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
    ) -> Operation<'context> {
        match (self, checked) {
            (Self::Add, true) => CAddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Add, false) => AddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Subtract, true) => CSubOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Subtract, false) => SubOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Multiply, true) => CMulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Multiply, false) => MulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Divide, true) => CDivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Divide, false) => DivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Remainder, _) => ModOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Exponentiation, true) => CExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            (Self::Exponentiation, false) => ExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
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
    /// type-annotated IR. Operands are evaluated right-to-left to match solc.
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

        let operation = operation.build(
            self.checked,
            self.state.builder.context,
            self.state.builder.unknown_location,
            lhs,
            rhs,
        );
        let value = block
            .append_operation(operation)
            .result(0)
            .expect("arithmetic operation produces one result")
            .into();
        Ok((value, block))
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

    /// Lowers a prefix step (`++x`, `--x`), yielding the value after the step.
    ///
    /// The other prefix operators (`!`, `~`, unary `-`, `delete`) are lowered
    /// by later domains.
    pub(super) fn emit_prefix(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            PrefixExpressionOperator::PlusPlus(_) => ArithmeticOperation::Add,
            PrefixExpressionOperator::MinusMinus(_) => ArithmeticOperation::Subtract,
            // `!` is a logical operator, lowered by its own domain.
            PrefixExpressionOperator::Bang(_) => {
                return self.emit_not(&expression.operand(), block);
            }
            _ => unimplemented!("prefix operator lowering"),
        };
        let (_old, new, block) =
            self.emit_increment_decrement(operation, &expression.operand(), block)?;
        Ok((new, block))
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
        let operation = operation.build(
            self.checked,
            self.state.builder.context,
            self.state.builder.unknown_location,
            old,
            one,
        );
        let new = block
            .append_operation(operation)
            .result(0)
            .expect("step operation produces one result")
            .into();
        self.emit_lvalue_store(&lvalue, new, &block);
        Ok((old, new, block))
    }
}
