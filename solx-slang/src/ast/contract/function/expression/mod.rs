//!
//! Expression lowering to MLIR SSA values.
//!

pub mod arithmetic;
pub mod assignment;
pub mod call;
pub mod logical;
pub mod operator;
pub mod storage;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::cst::NodeId;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, u64>,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, u64>,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
        }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns the SSA value produced and the continuation block (which may
    /// differ from the input block for short-circuit operators).
    /// # Errors
    ///
    /// Returns an error if the expression contains unsupported constructs.
    pub fn emit(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expression {
            Expression::DecimalNumberExpression(decimal) => {
                let literal = decimal.literal();
                let text = literal.text.as_str();
                let value = if let Ok(value) = text.parse::<i64>() {
                    self.state.builder.emit_sol_constant(value, &block)
                } else {
                    self.state
                        .builder
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
                    self.state.builder.emit_sol_constant(parsed, &block)
                } else {
                    self.state
                        .builder
                        .emit_sol_constant_from_hex_str(stripped, &block)?
                };
                Ok((value, block))
            }
            Expression::TrueKeyword => {
                let value = self.state.builder.emit_sol_constant(1, &block);
                Ok((value, block))
            }
            Expression::FalseKeyword => {
                let value = self.state.builder.emit_sol_constant(0, &block);
                Ok((value, block))
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        let slot = self
                            .storage_layout
                            .get(&state_variable.node_id())
                            .ok_or_else(|| {
                                anyhow::anyhow!("unregistered state variable: {name}")
                            })?;
                        let value = self.emit_storage_load(*slot, &block)?;
                        Ok((value, block))
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let pointer = self.environment.variable(&name).ok_or_else(|| {
                            anyhow::anyhow!("unregistered local variable: {name}")
                        })?;
                        let value = self.state.builder.emit_sol_load(pointer, &block)?;
                        Ok((value, block))
                    }
                    None => anyhow::bail!("unresolved identifier: {name}"),
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
                self.emit_comparison(&left, &right, &operator.text, block)
            }
            Expression::InequalityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_comparison(&left, &right, &operator.text, block)
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
                self::call::CallEmitter::new(self).emit_function_call(&callee, &arguments, block)
            }
            Expression::MemberAccessExpression(access) => {
                let operand = access.operand();
                let member = access.member().name();
                self::call::CallEmitter::new(self).emit_member_access(&operand, &member, block)
            }
            Expression::TupleExpression(tuple) => {
                let items = tuple.items();
                // TODO: support multi-value tuples (e.g. tuple deconstruction)
                anyhow::ensure!(items.len() == 1, "multi-value tuples not yet supported");
                let item = items.iter().next().expect("length checked to be 1 above");
                let inner = item
                    .expression()
                    .ok_or_else(|| anyhow::anyhow!("empty tuple element"))?;
                self.emit(&inner, block)
            }
            Expression::ExponentiationExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let (lhs, block) = self.emit(&left, block)?;
                let (rhs, block) = self.emit(&right, block)?;
                let result = self.state.builder.emit_binary_operation(
                    solx_mlir::Builder::SOL_EXP,
                    lhs,
                    rhs,
                    self.state.builder.get_type(solx_mlir::Builder::UI256),
                    &block,
                )?;
                Ok((result, block))
            }
            Expression::ConditionalExpression(conditional) => {
                let condition = conditional.operand();
                let (condition_value, block) = self.emit(&condition, block)?;
                let condition_boolean = self.emit_is_nonzero(condition_value, &block);

                let ui256 = self.state.builder.get_type(solx_mlir::Builder::UI256);
                let (then_block, else_block, result) =
                    self.state
                        .builder
                        .emit_scf_if(condition_boolean, ui256, &block)?;

                let true_expression = conditional.true_expression();
                let (then_value, then_end) = self.emit(&true_expression, then_block)?;
                self.state.builder.emit_scf_yield(&[then_value], &then_end);

                let false_expression = conditional.false_expression();
                let (else_value, else_end) = self.emit(&false_expression, else_block)?;
                self.state.builder.emit_scf_yield(&[else_value], &else_end);

                Ok((result, block))
            }
            _ => anyhow::bail!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits a `sol.cmp ne 0` producing `i1` from a `ui256`.
    ///
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let zero = self.state.builder.emit_sol_constant(0, block);
        self.state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Ne, block)
    }
}
