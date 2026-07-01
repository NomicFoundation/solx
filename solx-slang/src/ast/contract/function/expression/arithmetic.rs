//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::CmpPredicate;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a binary arithmetic Sol dialect operation.
    ///
    /// When `target_type` is `Some`, both operands are cast to that type and
    /// the result has that type (matching solc's type-annotated MLIR output).
    /// When `None`, selects the wider operand type by bit width.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: Operator,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (rhs, block) = self.emit_value(right, block)?;
        let (lhs, block) = self.emit_value(left, block)?;
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = AstType::new(lhs.r#type()).integer_bit_width();
            let rhs_width = AstType::new(rhs.r#type()).integer_bit_width();
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let lhs = TypeConversion::from_target_type(result_type, self.state).emit(
            lhs,
            self.state,
            &block,
        );
        let rhs = TypeConversion::from_target_type(result_type, self.state).emit(
            rhs,
            self.state,
            &block,
        );
        let value = block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.mlir_context,
                self.state.location(),
                lhs,
                rhs,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (old, _) = self.emit_increment_decrement(operand, operator, &block)?;
        Ok((old, block))
    }

    /// Emits prefix operators: `!`, `-`, `~`, `++`, `--`.
    ///
    /// When `target_type` is `Some`, unary operations use that type (matching
    /// solc's typed MLIR). When `None`, falls back to ui256 semantics.
    pub fn emit_prefix(
        &self,
        operator: Operator,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match operator {
            Operator::Increment | Operator::Decrement => {
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block)?;
                Ok((new_value, block))
            }
            Operator::BitwiseNot => {
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, self.state)
                    .emit(value, self.state, &block);
                let result = block
                    .append_operation(
                        NotOperation::builder(
                            self.state.mlir_context,
                            self.state.location(),
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
            Operator::Not => {
                let (value, block) = self.emit_value(operand, block)?;
                let zero =
                    AstValue::constant(0, AstType::new(value.r#type()), self.state, &block)
                        .into_mlir();
                let cmp = AstValue::new(value)
                    .compare(AstValue::new(zero), CmpPredicate::Eq, self.state, &block)
                    .into_mlir();
                let result_type = target_type.unwrap_or(
                    AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir(),
                );
                let result = TypeConversion::from_target_type(result_type, self.state)
                    .emit(cmp, self.state, &block);
                Ok((result, block))
            }
            Operator::Subtract => {
                // Unary negation uses unchecked subtraction. Checked negation
                // requires signed-type awareness (e.g. -INT_MIN should revert
                // in checked mode) which needs a dedicated op — not sol.csub,
                // since the operand may be in an unsigned literal type.
                let (value, block) = self.emit_value(operand, block)?;
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, self.state)
                    .emit(value, self.state, &block);
                let zero =
                    AstValue::constant(0, AstType::new(operand_type), self.state, &block)
                        .into_mlir();
                let result = block
                    .append_operation(
                        SubOperation::builder(
                            self.state.mlir_context,
                            self.state.location(),
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
            _ => anyhow::bail!("unsupported prefix operator: {operator:?}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)`.
    ///
    /// Handles both local variables and state variables via
    /// `resolve_to_definition()`.
    fn emit_increment_decrement(
        &self,
        operand: &Expression,
        operator: Operator,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, Value<'context, 'block>)> {
        let Expression::Identifier(identifier) = operand else {
            anyhow::bail!("unsupported operand for {operator:?}");
        };
        let name = identifier.name();

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .ok_or_else(|| anyhow::anyhow!("unregistered state variable: {name}"))?;
                let element_type = TypeConversion::resolve_state_variable_type(
                    &state_variable,
                    self.state,
                )?;
                let old = self.emit_storage_load(slot, element_type, block)?;
                let one =
                    AstValue::constant(1, AstType::new(element_type), self.state, block)
                        .into_mlir();
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.mlir_context,
                        self.state.location(),
                        old,
                        one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                self.emit_storage_store(slot, new_value, element_type, block);
                Ok((old, new_value))
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = Pointer::new(pointer)
                    .load(AstType::new(element_type), self.state, block)
                    .into_mlir();
                let typed_one =
                    AstValue::constant(1, AstType::new(element_type), self.state, block)
                        .into_mlir();
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.mlir_context,
                        self.state.location(),
                        old,
                        typed_one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                Pointer::new(pointer).store(AstValue::new(new_value), self.state, block);
                Ok((old, new_value))
            }
            None => anyhow::bail!("unresolved identifier: {name}"),
            Some(_) => anyhow::bail!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
