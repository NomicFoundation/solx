//!
//! Expression lowering to MLIR SSA values.
//!

pub mod arithmetic;
pub mod assignment;
pub mod call;
pub mod logical;
pub mod storage;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::Type;
use slang_solidity::cst::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ICmpPredicate;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment.
    environment: &'state Environment<'context, 'block>,
    /// The function region for creating new blocks.
    region: &'state Region<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, u64>,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        region: &'state Region<'context>,
        storage_layout: &'state HashMap<NodeId, u64>,
    ) -> Self {
        Self {
            state,
            environment,
            region,
            storage_layout,
        }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns the SSA value produced and the continuation block (which may
    /// differ from the input block for short-circuit operators).
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
                    self.state.builder().emit_sol_constant(value, &block)
                } else {
                    self.state
                        .builder()
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
                    self.state.builder().emit_sol_constant(parsed, &block)
                } else {
                    self.state
                        .builder()
                        .emit_sol_constant_from_hex_str(stripped, &block)?
                };
                Ok((value, block))
            }
            Expression::TrueKeyword => {
                let value = self.state.builder().emit_sol_constant(1, &block);
                Ok((value, block))
            }
            Expression::FalseKeyword => {
                let value = self.state.builder().emit_sol_constant(0, &block);
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
                        let value = self.emit_load(pointer, &block)?;
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
            // TODO: support ExponentiationExpression and ConditionalExpression
            _ => anyhow::bail!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Returns whether an expression has a signed integer type.
    ///
    /// Queries slang-solidity's semantic type information via `get_type()`.
    /// TODO: handle unknown type (binder panic) more precisely than defaulting to unsigned
    pub fn is_signed(expression: &Expression) -> bool {
        Self::expression_type(expression)
            .is_some_and(|t| matches!(t, Type::Integer(ref i) if i.signed()))
    }

    /// Emits a `sol.store` to a pointer via the builder.
    ///
    pub fn emit_store(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        self.state.builder().emit_sol_store(value, pointer, block);
    }

    /// Emits a `sol.alloca` for a local variable via the builder.
    ///
    pub fn emit_alloca(&self, block: &BlockRef<'context, 'block>) -> Value<'context, 'block> {
        self.state.builder().emit_sol_alloca(block)
    }

    /// Emits an `icmp ne 0` producing `i1` from an `i256`.
    ///
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let zero = self.state.builder().emit_sol_constant(0, block);
        self.state
            .builder()
            .emit_icmp(value, zero, ICmpPredicate::Ne, block)
    }

    /// Emits a `sol.load` from a pointer via the builder.
    ///
    fn emit_load(
        &self,
        pointer: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state.builder().emit_sol_load(pointer, block)
    }

    /// Emits a generic two-operand LLVM operation.
    ///
    fn emit_llvm_operation(
        &self,
        operation_name: &str,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        self.state.builder().emit_binary_operation(
            operation_name,
            lhs,
            rhs,
            self.state.i256(),
            block,
        )
    }

    /// Emits an EVM intrinsic via the builder.
    ///
    fn emit_intrinsic_call(
        &self,
        name: &str,
        operands: &[Value<'context, 'block>],
        has_result: bool,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<Value<'context, 'block>>> {
        self.state
            .builder()
            .emit_evm_intrinsic(name, operands, has_result, block)
    }

    /// Returns the semantic type of an expression, if available.
    ///
    /// The `Expression` enum does not expose a uniform `get_type()` method,
    /// so this dispatches to each variant's inner struct.
    ///
    /// Wraps the call in `catch_unwind` because slang-solidity's
    /// `Binder::node_typing()` panics on nodes without typing info
    /// instead of returning `None`.
    fn expression_type(expression: &Expression) -> Option<Type> {
        /// Calls `get_type()` on a slang AST node, returning `None` if the
        /// binder panics due to missing typing information.
        /// TODO: remove catch_unwind once slang-solidity's `Binder::node_typing()` returns `Option`
        fn try_get_type<F>(get_type: F) -> Option<Type>
        where
            F: FnOnce() -> Option<Type> + std::panic::UnwindSafe,
        {
            std::panic::catch_unwind(get_type).ok().flatten()
        }

        match expression {
            Expression::AssignmentExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::ConditionalExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::OrExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::AndExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::EqualityExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::InequalityExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::BitwiseOrExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::BitwiseXorExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::BitwiseAndExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::ShiftExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::AdditiveExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::MultiplicativeExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::ExponentiationExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::PostfixExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::PrefixExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::FunctionCallExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::MemberAccessExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::IndexAccessExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::NewExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::TupleExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::HexNumberExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::DecimalNumberExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::Identifier(inner) => try_get_type(|| inner.get_type()),
            Expression::CallOptionsExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::TypeExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::ArrayExpression(inner) => try_get_type(|| inner.get_type()),
            Expression::StringExpression(_)
            | Expression::ElementaryType(_)
            | Expression::PayableKeyword
            | Expression::ThisKeyword
            | Expression::SuperKeyword
            | Expression::TrueKeyword
            | Expression::FalseKeyword => None,
        }
    }
}
