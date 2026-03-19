//!
//! Expression lowering to MLIR SSA values.
//!

pub(crate) mod arithmetic;
pub(crate) mod call;
pub(crate) mod comparison;
pub(crate) mod storage;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ICmpPredicate;

/// Lowers Solidity expressions to MLIR SSA values.
pub(crate) struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment.
    environment: &'state Environment<'context, 'block>,
    /// The function region for creating new blocks.
    region: &'state Region<'context>,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub(crate) fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        region: &'state Region<'context>,
    ) -> Self {
        Self {
            state,
            environment,
            region,
        }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns the SSA value produced and the continuation block (which may
    /// differ from the input block for short-circuit operators).
    pub(crate) fn emit(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expression {
            Expression::DecimalNumberExpression(decimal) => {
                let literal = decimal.literal();
                let text = literal.text.as_str();
                let value = if let Ok(value) = text.parse::<i64>() {
                    self.state.emit_sol_constant(value, &block)
                } else {
                    self.state
                        .emit_sol_constant_from_decimal_str(text, &block)?
                };
                Ok((value, block))
            }
            Expression::HexNumberExpression(hex) => {
                let literal = hex.literal();
                let text = literal.text.as_str();
                let stripped = text
                    .strip_prefix("0x")
                    .or(text.strip_prefix("0X"))
                    .unwrap_or(text);
                let value = if let Ok(parsed) = i64::from_str_radix(stripped, 16) {
                    self.state.emit_sol_constant(parsed, &block)
                } else {
                    self.state
                        .emit_sol_constant_from_hex_str(stripped, &block)?
                };
                Ok((value, block))
            }
            Expression::TrueKeyword => {
                let value = self.state.emit_sol_constant(1, &block);
                Ok((value, block))
            }
            Expression::FalseKeyword => {
                let value = self.state.emit_sol_constant(0, &block);
                Ok((value, block))
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(_)) => {
                        // TODO: compute slot offsets instead of deriving from names
                        let slot = self.state.state_variable_slot(&name).ok_or_else(|| {
                            anyhow::anyhow!("unregistered state variable: {name}")
                        })?;
                        let value = self.emit_storage_load(slot, &block)?;
                        Ok((value, block))
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let pointer = self.environment.variable(&name).ok_or_else(|| {
                            anyhow::anyhow!("unregistered local variable: {name}")
                        })?;
                        let value = self.emit_load(pointer, &block)?;
                        Ok((value, block))
                    }
                    None => {
                        // Fallback for identifiers the binder cannot resolve.
                        // TODO: check if slang-solidity can resolve all identifier references so that this fallback is not needed
                        if let Some(pointer) = self.environment.variable(&name) {
                            let value = self.emit_load(pointer, &block)?;
                            Ok((value, block))
                        } else if let Some(slot) = self.state.state_variable_slot(&name) {
                            let value = self.emit_storage_load(slot, &block)?;
                            Ok((value, block))
                        } else {
                            anyhow::bail!("undefined variable: {name}")
                        }
                    }
                    Some(_) => anyhow::bail!("unsupported identifier reference: {name}"),
                }
            }
            Expression::AssignmentExpression(assign) => self.emit_assignment(assign, block),
            Expression::AdditiveExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::MultiplicativeExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::EqualityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_icmp(&left, &right, &operator.text, block)
            }
            Expression::InequalityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_icmp(&left, &right, &operator.text, block)
            }
            Expression::AndExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_and(&left, &right, block)
            }
            Expression::OrExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_or(&left, &right, block)
            }
            Expression::PostfixExpression(expression) => {
                let operand = expression.operand();
                let operator = expression.operator();
                self.emit_postfix(&operand, &operator.text, block)
            }
            Expression::PrefixExpression(expression) => {
                let operator = expression.operator();
                let operand = expression.operand();
                self.emit_prefix(&operator.text, &operand, block)
            }
            Expression::BitwiseAndExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "&", block)
            }
            Expression::BitwiseOrExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "|", block)
            }
            Expression::BitwiseXorExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "^", block)
            }
            Expression::ShiftExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::FunctionCallExpression(call) => {
                let callee = call.operand();
                let arguments = call.arguments();
                self.emit_function_call(&callee, &arguments, block)
            }
            Expression::MemberAccessExpression(access) => {
                let operand = access.operand();
                let member = access.member().name();
                self.emit_member_access(&operand, &member, block)
            }
            _ => anyhow::bail!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits a `sol.store` to a pointer via the builder.
    ///
    /// TODO: remove this thin wrapper and call directly
    pub(crate) fn emit_store(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        self.state.emit_sol_store(value, pointer, block);
    }

    /// Emits a `sol.alloca` for a local variable via the builder.
    ///
    /// TODO: remove this thin wrapper and call directly
    pub(crate) fn emit_alloca(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        self.state.emit_sol_alloca(block)
    }

    /// Emits an `icmp ne 0` producing `i1` from an `i256`.
    ///
    /// TODO: remove this thin wrapper and call directly
    pub(crate) fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let zero = self.state.emit_sol_constant(0, block);
        self.state.emit_icmp(value, zero, ICmpPredicate::Ne, block)
    }

    /// Emits a `sol.load` from a pointer via the builder.
    ///
    /// TODO: remove this thin wrapper and call directly
    fn emit_load(
        &self,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state.emit_sol_load(pointer, block)
    }

    /// Emits a generic two-operand LLVM operation.
    ///
    /// TODO: remove this thin wrapper and call directly
    fn emit_llvm_operation(
        &self,
        operation_name: &str,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state
            .emit_llvm_operation(operation_name, lhs, rhs, self.state.i256(), block)
    }

    /// Emits an EVM intrinsic via the builder.
    ///
    /// TODO: remove this thin wrapper and call directly
    fn emit_intrinsic_call(
        &self,
        name: &str,
        operands: &[Value<'context, 'block>],
        has_result: bool,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<Value<'context, 'block>>> {
        self.state
            .emit_evm_intrinsic(name, operands, has_result, block)
    }

    /// Returns whether an expression has a signed integer type.
    ///
    /// Propagates signedness through arithmetic, shift, prefix, postfix,
    /// and assignment expressions so that operations like `/`, `%`, and
    /// `>>` select the correct signed LLVM operation.
    ///
    /// TODO: check if slang-solidity can provide this information instead of re-deriving it here
    fn is_signed_expression(&self, expression: &Expression) -> bool {
        match expression {
            Expression::Identifier(identifier) => self.environment.is_signed(&identifier.name()),
            Expression::PrefixExpression(prefix_expression)
                if prefix_expression.operator().text == "-" =>
            {
                true
            }
            Expression::PrefixExpression(prefix_expression) => {
                self.is_signed_expression(&prefix_expression.operand())
            }
            Expression::AdditiveExpression(expr) => {
                self.is_signed_expression(&expr.left_operand())
                    || self.is_signed_expression(&expr.right_operand())
            }
            Expression::MultiplicativeExpression(expr) => {
                self.is_signed_expression(&expr.left_operand())
                    || self.is_signed_expression(&expr.right_operand())
            }
            Expression::ShiftExpression(expr) => self.is_signed_expression(&expr.left_operand()),
            Expression::PostfixExpression(expr) => self.is_signed_expression(&expr.operand()),
            Expression::AssignmentExpression(expr) => {
                self.is_signed_expression(&expr.left_operand())
            }
            _ => false,
        }
    }
}
