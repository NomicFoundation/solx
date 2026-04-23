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
use std::rc::Rc;

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::SemanticAnalysis;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::cst::NodeId;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;

use self::call::type_conversion::TypeConversion;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// Slang semantic analysis for resolving expression types.
    pub semantic: Rc<SemanticAnalysis>,
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, u64>,
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default (Solidity 0.8+). Set to `false` inside `unchecked {}`
    /// blocks and for-loop step expressions.
    pub checked: bool,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        semantic: &Rc<SemanticAnalysis>,
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, u64>,
        checked: bool,
    ) -> Self {
        Self {
            semantic: Rc::clone(semantic),
            state,
            environment,
            storage_layout,
            checked,
        }
    }

    /// Emits MLIR for an expression that must produce a value.
    ///
    /// Delegates to [`Self::emit`] and returns an error for void expressions
    /// (e.g. calls to functions with no return value).
    pub fn emit_value(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(expression, block)?;
        let value = value.ok_or_else(|| anyhow::anyhow!("expression produced no value"))?;
        Ok((value, block))
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns `None` for void expressions (calls with no return value).
    /// Use [`Self::emit_value`] when a value is required.
    ///
    /// # Errors
    ///
    /// Returns an error if the expression contains unsupported constructs.
    pub fn emit(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        match expression {
            Expression::DecimalNumberExpression(decimal) => {
                let literal = decimal.literal();
                let text = literal.text.as_str();
                // TODO: use resolve_expression_type once slang provides the
                // implicit conversion target type for literals.
                let digit_count = text.chars().filter(|char| *char != '_').count() as u32;
                let approx_bits = (digit_count * 3322).div_ceil(1000);
                let bit_width = Self::round_up_to_solidity_width(approx_bits);
                let min_type =
                    Type::from(IntegerType::unsigned(self.state.builder.context, bit_width));
                let value = self
                    .state
                    .builder
                    .emit_sol_constant_from_decimal_str(text, min_type, &block)?;
                Ok((Some(value), block))
            }
            Expression::HexNumberExpression(hex) => {
                let literal = hex.literal();
                let text = literal.text.as_str();
                let stripped = text
                    .strip_prefix("0x")
                    .or(text.strip_prefix("0X"))
                    .unwrap_or(text);
                // TODO: use resolve_expression_type once slang provides the
                // implicit conversion target type for literals.
                let significant_digits = stripped.trim_start_matches('0').len() as u32;
                let bit_width = Self::round_up_to_solidity_width(significant_digits * 4);
                let min_type =
                    Type::from(IntegerType::unsigned(self.state.builder.context, bit_width));
                let value = self
                    .state
                    .builder
                    .emit_sol_constant_from_hex_str(stripped, min_type, &block)?;
                Ok((Some(value), block))
            }
            Expression::TrueKeyword => {
                let value = self.state.builder.emit_arith_constant_bool(true, &block);
                Ok((Some(value), block))
            }
            Expression::FalseKeyword => {
                let value = self.state.builder.emit_arith_constant_bool(false, &block);
                Ok((Some(value), block))
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
                        Ok((Some(value), block))
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) =
                            self.environment.variable_with_type(&name).ok_or_else(|| {
                                anyhow::anyhow!("unregistered local variable: {name}")
                            })?;
                        let value =
                            self.state
                                .builder
                                .emit_sol_load(pointer, element_type, &block)?;
                        Ok((Some(value), block))
                    }
                    None => anyhow::bail!("unresolved identifier: {name}"),
                    Some(_) => anyhow::bail!("unsupported identifier reference: {name}"),
                }
            }
            Expression::AssignmentExpression(assign) => self
                .emit_assignment(assign, block)
                .map(|(value, block)| (Some(value), block)),
            Expression::AdditiveExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::MultiplicativeExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ExponentiationExpression(expression) => {
                let target_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "**", target_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::EqualityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_comparison(&left, &right, &operator.text, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::InequalityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_comparison(&left, &right, &operator.text, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::AndExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_and(&left, &right, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::OrExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_or(&left, &right, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::PostfixExpression(expression) => {
                let operand = expression.operand();
                let operator = expression.operator();
                self.emit_postfix(&operand, &operator.text, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::PrefixExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let operator = expression.operator();
                let operand = expression.operand();
                self.emit_prefix(&operator.text, &operand, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseAndExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "&", result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseOrExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "|", result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseXorExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, "^", result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ShiftExpression(expression) => {
                let result_type = self.resolve_expression_type(expression.node_id());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = expression.operator();
                self.emit_binary_op(&left, &right, &operator.text, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::FunctionCallExpression(call) => {
                self::call::CallEmitter::new(self).emit_function_call(call, block)
            }
            Expression::MemberAccessExpression(access) => {
                let member = access.member();
                self::call::CallEmitter::new(self)
                    .emit_member_access(&member, block)
                    .map(|(value, block)| (Some(value), block))
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
            Expression::ConditionalExpression(conditional) => {
                let result_type = self
                    .resolve_expression_type(conditional.node_id())
                    .unwrap_or_else(|| self.state.builder.types.ui256);
                let condition = conditional.operand();
                let (condition_value, block) = self.emit_value(&condition, block)?;
                let condition_boolean = self.emit_is_nonzero(condition_value, &block);

                let (then_block, else_block, result) =
                    self.state
                        .builder
                        .emit_scf_if(condition_boolean, result_type, &block)?;

                let true_expression = conditional.true_expression();
                let (then_value, then_end) = self.emit_value(&true_expression, then_block)?;
                let then_cast = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(then_value, &self.state.builder, &then_end);
                self.state.builder.emit_scf_yield(&[then_cast], &then_end);

                let false_expression = conditional.false_expression();
                let (else_value, else_end) = self.emit_value(&false_expression, else_block)?;
                let else_cast = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(else_value, &self.state.builder, &else_end);
                self.state.builder.emit_scf_yield(&[else_cast], &else_end);

                Ok((Some(result), block))
            }
            _ => anyhow::bail!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits a `sol.cmp ne 0` producing `i1` from a value.
    ///
    /// Short-circuits when the value is already `i1` (e.g. from `sol.cmp`),
    /// avoiding the redundant `sol.cmp ne, %i1, %zero_i1 : i1` pattern.
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        if solx_mlir::TypeFactory::integer_bit_width(value.r#type()) == 1 {
            return value;
        }
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, value.r#type(), block);
        self.state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Ne, block)
    }

    /// Resolves the Solidity type of an expression node to an MLIR type.
    ///
    /// Returns `None` when the semantic analysis has no type info for the node.
    /// Panics on types that `resolve_slang_type` does not yet handle.
    pub fn resolve_expression_type(&self, node_id: NodeId) -> Option<Type<'context>> {
        let slang_type = self.semantic.get_type_from_node_id(node_id)?;
        Some(TypeConversion::resolve_slang_type(
            &slang_type,
            self.state.builder.context,
            &self.state.builder,
        ))
    }

    /// Rounds a bit count up to the nearest Solidity integer width.
    ///
    /// Solidity supports unsigned integers in 8-bit increments: 8, 16, 24, ..., 256.
    /// Returns 8 for zero bits (smallest Solidity type).
    fn round_up_to_solidity_width(bits: u32) -> u32 {
        if bits == 0 {
            return solx_utils::BIT_LENGTH_BYTE as u32;
        }
        let rounded =
            bits.div_ceil(solx_utils::BIT_LENGTH_BYTE as u32) * solx_utils::BIT_LENGTH_BYTE as u32;
        rounded.min(solx_utils::BIT_LENGTH_FIELD as u32)
    }
}
