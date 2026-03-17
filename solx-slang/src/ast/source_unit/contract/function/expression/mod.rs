//!
//! Expression lowering to MLIR SSA values.
//!

mod arithmetic;
mod call;
mod comparison;
mod storage;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::Environment;
use solx_mlir::ICmpPredicate;
use solx_mlir::MlirContext;

/// Lowers Solidity expressions to MLIR SSA values.
pub(crate) struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state MlirContext<'context>,
    /// Variable environment.
    environment: &'state Environment<'context, 'block>,
    /// The function region for creating new blocks.
    region: &'state Region<'context>,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Gas stipend for `address.transfer()` and `address.send()` calls.
    const TRANSFER_GAS_STIPEND: i64 = 2300;

    /// Creates a new expression emitter.
    pub(crate) fn new(
        state: &'state MlirContext<'context>,
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
        expr: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expr {
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
                        let slot = self.state.state_variable_slot(&name).ok_or_else(|| {
                            anyhow::anyhow!("unregistered state variable: {name}")
                        })?;
                        let value = self.emit_storage_load(slot, &block)?;
                        Ok((value, block))
                    }
                    Some(
                        Definition::Variable(_)
                        | Definition::Parameter(_)
                        | Definition::TypeParameter(_),
                    ) => {
                        let pointer = self.environment.variable(&name).ok_or_else(|| {
                            anyhow::anyhow!("unregistered local variable: {name}")
                        })?;
                        let value = self.emit_load(pointer, &block)?;
                        Ok((value, block))
                    }
                    None => {
                        // Fallback for identifiers the binder cannot resolve.
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
            Expression::AdditiveExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::MultiplicativeExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_binary_op(&left, &right, &operator.text, block)
            }
            Expression::EqualityExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_icmp(&left, &right, &operator.text, block)
            }
            Expression::InequalityExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
                self.emit_icmp(&left, &right, &operator.text, block)
            }
            Expression::AndExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_and(&left, &right, block)
            }
            Expression::OrExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_or(&left, &right, block)
            }
            Expression::PostfixExpression(expr) => {
                let operand = expr.operand();
                let operator = expr.operator();
                self.emit_postfix(&operand, &operator.text, block)
            }
            Expression::PrefixExpression(expr) => {
                let operator = expr.operator();
                let operand = expr.operand();
                self.emit_prefix(&operator.text, &operand, block)
            }
            Expression::BitwiseAndExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_binary_op(&left, &right, "&", block)
            }
            Expression::BitwiseOrExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_binary_op(&left, &right, "|", block)
            }
            Expression::BitwiseXorExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                self.emit_binary_op(&left, &right, "^", block)
            }
            Expression::ShiftExpression(expr) => {
                let left = expr.left_operand();
                let right = expr.right_operand();
                let operator = expr.operator();
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
            _ => anyhow::bail!("unsupported expression: {:?}", std::mem::discriminant(expr)),
        }
    }

    /// Emits a `sol.store` to a pointer via the builder.
    pub(crate) fn emit_store(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        self.state.emit_sol_store(value, pointer, block);
    }

    /// Emits a `sol.alloca` for a local variable via the builder.
    pub(crate) fn emit_alloca(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        self.state.emit_sol_alloca(block)
    }

    /// Emits an `icmp ne 0` producing `i1` from an `i256`.
    pub(crate) fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let zero = self.state.emit_sol_constant(0, block);
        self.state.emit_icmp(value, zero, ICmpPredicate::Ne, block)
    }

    /// Emits a `sol.load` from a pointer via the builder.
    fn emit_load(
        &self,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state.emit_sol_load(pointer, block)
    }

    /// Emits a generic two-operand LLVM operation.
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

    /// Returns whether an expression has a signed integer type.
    fn is_signed_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Identifier(id) => self.environment.is_signed(&id.name()),
            Expression::PrefixExpression(p) if p.operator().text == "-" => true,
            _ => false,
        }
    }

    /// Emits an EVM intrinsic via the builder.
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
}
