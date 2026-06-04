//!
//! Arithmetic expression lowering: binary operators, prefix, postfix.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PrefixExpression;

use solx_mlir::CmpPredicate;
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
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// A binary arithmetic operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithmeticOperation {
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
    /// Builds the Sol dialect operation for this operator.
    ///
    /// When `checked` is true, addition, subtraction, multiplication,
    /// division, and exponentiation use the checked variants (`sol.cadd` …);
    /// modulo is always unchecked. The result type is inferred from `lhs`
    /// (`SameOperandsAndResultType`).
    pub fn emit_operation<'context>(
        self,
        checked: bool,
        context: &'context melior::Context,
        location: Location<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        match self {
            Self::Add if checked => CAddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Add => AddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Subtract if checked => CSubOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Subtract => SubOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Multiply if checked => CMulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Multiply => MulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Divide if checked => CDivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Divide => DivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Remainder => ModOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Exponentiation if checked => CExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Exponentiation => ExpOperation::builder(context, location)
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
    pub fn emit_additive(
        &self,
        expression: &AdditiveExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            ast::AdditiveExpressionOperator::Plus(_) => ArithmeticOperation::Add,
            ast::AdditiveExpressionOperator::Minus(_) => ArithmeticOperation::Subtract,
        };
        self.emit_arithmetic(
            &expression.left_operand(),
            &expression.right_operand(),
            operation,
            self.resolve_slang_type(expression.get_type()),
            block,
        )
    }

    /// Lowers a multiplicative expression (`*`, `/`, `%`).
    pub fn emit_multiplicative(
        &self,
        expression: &MultiplicativeExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            ast::MultiplicativeExpressionOperator::Asterisk(_) => ArithmeticOperation::Multiply,
            ast::MultiplicativeExpressionOperator::Slash(_) => ArithmeticOperation::Divide,
            ast::MultiplicativeExpressionOperator::Percent(_) => ArithmeticOperation::Remainder,
        };
        self.emit_arithmetic(
            &expression.left_operand(),
            &expression.right_operand(),
            operation,
            self.resolve_slang_type(expression.get_type()),
            block,
        )
    }

    /// Lowers an exponentiation expression (`**`).
    pub fn emit_exponentiation(
        &self,
        expression: &ExponentiationExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        self.emit_arithmetic(
            &expression.left_operand(),
            &expression.right_operand(),
            ArithmeticOperation::Exponentiation,
            self.resolve_slang_type(expression.get_type()),
            block,
        )
    }

    /// Emits a binary arithmetic operation over two operand expressions.
    fn emit_arithmetic(
        &self,
        left: &Expression,
        right: &Expression,
        operation: ArithmeticOperation,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, target_type, block)?;
        let value = block
            .append_operation(operation.emit_operation(
                self.checked,
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

    /// Evaluates both operands (right then left, matching solc's evaluation
    /// order), then casts each to the common result type.
    ///
    /// When `target_type` is `Some`, both operands and the result take that
    /// type; when `None`, the wider operand type by bit width is used.
    pub fn emit_binary_operands(
        &self,
        left: &Expression,
        right: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (rhs, block) = self.emit_value(right, block)?;
        let (lhs, block) = self.emit_value(left, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = solx_mlir::TypeFactory::integer_bit_width(lhs.r#type());
            let rhs_width = solx_mlir::TypeFactory::integer_bit_width(rhs.r#type());
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
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

    /// Lowers a postfix expression (`x++`, `x--`), yielding the old value.
    pub fn emit_postfix(
        &self,
        expression: &PostfixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            ast::PostfixExpressionOperator::PlusPlus(_) => ArithmeticOperation::Add,
            ast::PostfixExpressionOperator::MinusMinus(_) => ArithmeticOperation::Subtract,
        };
        let (old, _) = self.emit_step(&expression.operand(), operation, &block)?;
        Ok((old, block))
    }

    /// Lowers a prefix expression (`!`, `-`, `~`, `++x`, `--x`).
    pub fn emit_prefix(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let target_type = self.resolve_slang_type(expression.get_type());
        let operand = expression.operand();
        match expression.operator() {
            ast::PrefixExpressionOperator::PlusPlus(_) => {
                let (_old, new_value) =
                    self.emit_step(&operand, ArithmeticOperation::Add, &block)?;
                Ok((new_value, block))
            }
            ast::PrefixExpressionOperator::MinusMinus(_) => {
                let (_old, new_value) =
                    self.emit_step(&operand, ArithmeticOperation::Subtract, &block)?;
                Ok((new_value, block))
            }
            ast::PrefixExpressionOperator::Bang(_) => {
                self.emit_logical_not(&operand, target_type, block)
            }
            ast::PrefixExpressionOperator::Tilde(_) => {
                self.emit_bitwise_not(&operand, target_type, block)
            }
            ast::PrefixExpressionOperator::Minus(_) => {
                self.emit_negate(&operand, target_type, block)
            }
            ast::PrefixExpressionOperator::DeleteKeyword(_) => {
                unimplemented!("the `delete` operator is not yet supported")
            }
        }
    }

    /// Emits logical negation (`!`) as `value == 0`.
    fn emit_logical_not(
        &self,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_value(operand, block)?;
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, value.r#type(), &block);
        let comparison = self
            .state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Eq, &block);
        let result_type = target_type.unwrap_or(self.state.builder.types.ui256);
        let result = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            comparison,
            &self.state.builder,
            &block,
        );
        Ok((result, block))
    }

    /// Emits bitwise negation (`~`) via `sol.not`.
    fn emit_bitwise_not(
        &self,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_value(operand, block)?;
        let operand_type = target_type.unwrap_or_else(|| value.r#type());
        let value = TypeConversion::from_target_type(operand_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        let result = block
            .append_operation(
                NotOperation::builder(
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                )
                .value(value)
                .build()
                .into(),
            )
            .result(0)
            .expect("sol.not always produces one result")
            .into();
        Ok((result, block))
    }

    /// Emits arithmetic negation (`-`) as unchecked `0 - value`.
    ///
    /// Checked negation would require signed-type awareness (e.g. `-INT_MIN`
    /// reverting), which needs a dedicated op rather than `sol.csub`, since the
    /// operand may carry an unsigned literal type.
    fn emit_negate(
        &self,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_value(operand, block)?;
        let operand_type = target_type.unwrap_or_else(|| value.r#type());
        let value = TypeConversion::from_target_type(operand_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, operand_type, &block);
        let result = block
            .append_operation(
                SubOperation::builder(
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                )
                .lhs(zero)
                .rhs(value)
                .build()
                .into(),
            )
            .result(0)
            .expect("sol.sub always produces one result")
            .into();
        Ok((result, block))
    }

    /// Loads, applies `+ 1` or `- 1`, stores, and returns `(old, new)`.
    ///
    /// Handles both local variables and state variables, resolved through the
    /// binder.
    fn emit_step(
        &self,
        operand: &Expression,
        operation: ArithmeticOperation,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Value<'context, 'block>)> {
        let Expression::Identifier(identifier) = operand else {
            unimplemented!("increment/decrement of a non-identifier operand is not yet supported");
        };
        let name = identifier.name();
        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("every state variable has a storage slot");
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    &self.state.builder,
                );
                let old = self.emit_storage_load(slot, element_type, block)?;
                let one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = block
                    .append_operation(operation.emit_operation(
                        self.checked,
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                        old,
                        one,
                    ))
                    .result(0)
                    .expect("a binary operation always produces one result")
                    .into();
                self.emit_storage_store(slot, new_value, element_type, block);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = self
                    .state
                    .builder
                    .emit_sol_load(pointer, element_type, block)?;
                let one = self.state.builder.emit_sol_constant(1, element_type, block);
                let new_value = block
                    .append_operation(operation.emit_operation(
                        self.checked,
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                        old,
                        one,
                    ))
                    .result(0)
                    .expect("a binary operation always produces one result")
                    .into();
                self.state.builder.emit_sol_store(new_value, pointer, block);
                Ok((old, new_value))
            }
            None => unreachable!("slang resolves every identifier reference"),
            Some(_) => {
                unimplemented!("increment/decrement of '{name}' is not yet supported")
            }
        }
    }
}
