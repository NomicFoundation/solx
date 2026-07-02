//!
//! Expression lowering to MLIR SSA values.
//!

pub mod access;
pub mod arithmetic;
pub mod assignment;
pub mod call;
pub mod logical;
pub mod member;
pub mod operator;
pub mod storage;

use std::collections::HashMap;

use melior::ir::BlockRef;
use operator::Operator;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Effect;
use solx_mlir::Environment;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;
use solx_utils::DataLocation;

use self::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default (Solidity 0.8+). Set to `false` inside `unchecked {}`
    /// blocks and for-loop step expressions.
    pub checked: bool,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        checked: bool,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            checked,
        }
    }

    /// Emits MLIR for an expression that must produce a value.
    ///
    /// Delegates to [`Self::emit`] and returns an error for void expressions.
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
    /// Returns `None` for void expressions. Use [`Self::emit_value`] when a
    /// value is required.
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
            Expression::DecimalNumberExpression(decimal_number) => {
                let value = decimal_number.integer_value().ok_or_else(|| {
                    anyhow::anyhow!(
                        "decimal literal cannot be lowered: it must evaluate to an integer \
                         after applying any units"
                    )
                })?;
                let result_type = self
                    .resolve_slang_type(decimal_number.get_type())
                    .expect("binder types every decimal literal node");
                let constant = Value::constant_from_bigint(&value, result_type, self.state, &block);
                Ok((Some(constant), block))
            }
            Expression::HexNumberExpression(hex_number) => {
                let value = hex_number
                    .integer_value()
                    .expect("hex literals always evaluate to integers");
                let result_type = self
                    .resolve_slang_type(hex_number.get_type())
                    .expect("binder types every hex literal node");
                let constant = Value::constant_from_bigint(&value, result_type, self.state, &block);
                Ok((Some(constant), block))
            }
            Expression::TrueKeyword(_) => {
                let constant = Value::boolean(true, self.state, &block);
                Ok((Some(constant), block))
            }
            Expression::FalseKeyword(_) => {
                let constant = Value::boolean(false, self.state, &block);
                Ok((Some(constant), block))
            }
            Expression::ThisKeyword(_) => {
                let contract_type = self
                    .state
                    .current_contract_type
                    .ok_or_else(|| anyhow::anyhow!("sol.this emitted outside a contract"))?;
                let value = Value::this(contract_type, self.state, &block);
                Ok((Some(value), block))
            }
            Expression::StringExpression(string_expression) => {
                let bytes = string_expression.value();
                let text = std::str::from_utf8(&bytes).expect("string literal is valid UTF-8");
                let value = Value::string_literal(text, self.state, &block);
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
                        let declared_type = state_variable.get_type().ok_or_else(|| {
                            anyhow::anyhow!("unresolved type for state variable: {name}")
                        })?;
                        let element_type =
                            TypeConversion::resolve_slang_type(&declared_type, None, self.state);
                        let address = Place::addr_of(
                            &slot.name,
                            Self::address_type(
                                self.state,
                                element_type,
                                DataLocation::Storage,
                                &declared_type,
                            ),
                            self.state,
                            &block,
                        );
                        let value = address.load(element_type, self.state, &block);
                        Ok((Some(value), block))
                    }
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        let value = pointer.load(element_type, self.state, &block);
                        Ok((Some(value), block))
                    }
                    Some(Definition::Constant(constant)) => {
                        let initializer = constant
                            .value()
                            .ok_or_else(|| anyhow::anyhow!("constant {name} has no initializer"))?;
                        self.emit(&initializer, block)
                    }
                    None => anyhow::bail!("unresolved identifier: {name}"),
                    Some(_) => anyhow::bail!("unsupported identifier reference: {name}"),
                }
            }
            Expression::AssignmentExpression(assign) => self
                .emit_assignment(assign, block)
                .map(|(value, block)| (Some(value), block)),
            Expression::AdditiveExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = match expression.operator() {
                    ast::AdditiveExpressionOperator::Plus(_) => Operator::Add,
                    ast::AdditiveExpressionOperator::Minus(_) => Operator::Subtract,
                };
                self.emit_binary_op(&left, &right, operator, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::MultiplicativeExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = match expression.operator() {
                    ast::MultiplicativeExpressionOperator::Asterisk(_) => Operator::Multiply,
                    ast::MultiplicativeExpressionOperator::Percent(_) => Operator::Remainder,
                    ast::MultiplicativeExpressionOperator::Slash(_) => Operator::Divide,
                };
                self.emit_binary_op(&left, &right, operator, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ExponentiationExpression(expression) => {
                let target_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::Exponentiation, target_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::EqualityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let predicate = match expression.operator() {
                    ast::EqualityExpressionOperator::BangEqual(_) => CmpPredicate::Ne,
                    ast::EqualityExpressionOperator::EqualEqual(_) => CmpPredicate::Eq,
                };
                self.emit_comparison(&left, &right, predicate, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::InequalityExpression(expression) => {
                let left = expression.left_operand();
                let right = expression.right_operand();
                let predicate = match expression.operator() {
                    ast::InequalityExpressionOperator::GreaterThan(_) => CmpPredicate::Gt,
                    ast::InequalityExpressionOperator::GreaterThanEqual(_) => CmpPredicate::Ge,
                    ast::InequalityExpressionOperator::LessThan(_) => CmpPredicate::Lt,
                    ast::InequalityExpressionOperator::LessThanEqual(_) => CmpPredicate::Le,
                };
                self.emit_comparison(&left, &right, predicate, block)
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
                let operator = match expression.operator() {
                    ast::PostfixExpressionOperator::MinusMinus(_) => Operator::Decrement,
                    ast::PostfixExpressionOperator::PlusPlus(_) => Operator::Increment,
                };
                self.emit_postfix(&operand, operator, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::PrefixExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let operator = match expression.operator() {
                    ast::PrefixExpressionOperator::Bang(_) => Operator::Not,
                    ast::PrefixExpressionOperator::DeleteKeyword(_) => Operator::Delete,
                    ast::PrefixExpressionOperator::Minus(_) => Operator::Subtract,
                    ast::PrefixExpressionOperator::MinusMinus(_) => Operator::Decrement,
                    ast::PrefixExpressionOperator::PlusPlus(_) => Operator::Increment,
                    ast::PrefixExpressionOperator::Tilde(_) => Operator::BitwiseNot,
                };
                let operand = expression.operand();
                self.emit_prefix(operator, &operand, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseAndExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::BitwiseAnd, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseOrExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::BitwiseOr, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseXorExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::BitwiseXor, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ShiftExpression(expression) => {
                let result_type = self.resolve_slang_type(expression.get_type());
                let left = expression.left_operand();
                let right = expression.right_operand();
                let operator = match expression.operator() {
                    ast::ShiftExpressionOperator::GreaterThanGreaterThan(_) => Operator::ShiftRight,
                    ast::ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => {
                        Operator::ShiftRight
                    }
                    ast::ShiftExpressionOperator::LessThanLessThan(_) => Operator::ShiftLeft,
                };
                self.emit_binary_op(&left, &right, operator, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::FunctionCallExpression(call) => {
                self::call::CallEmitter::new(self).emit_function_call(call, block)
            }
            Expression::TupleExpression(tuple) => {
                let items = tuple.items();
                // TODO: support multi-value tuple expressions.
                anyhow::ensure!(items.len() == 1, "multi-value tuples not yet supported");
                let item = items.iter().next().expect("length checked to be 1 above");
                let inner = item
                    .expression()
                    .ok_or_else(|| anyhow::anyhow!("empty tuple element"))?;
                self.emit(&inner, block)
            }
            Expression::ConditionalExpression(conditional) => {
                let result_type = self
                    .resolve_slang_type(conditional.get_type())
                    .unwrap_or_else(|| {
                        Type::unsigned(self.state.melior, solx_utils::BIT_LENGTH_FIELD)
                    });
                let condition = conditional.operand();
                let (condition_value, block) = self.emit_value(&condition, block)?;
                let condition_boolean = self.emit_is_nonzero(condition_value, &block);

                let result_slot = Place::stack(result_type, self.state, &block);
                let (then_block, else_block) =
                    Effect::new(self.state, block).branch(condition_boolean);

                let true_expression = conditional.true_expression();
                let (then_value, then_end) = self.emit_value(&true_expression, then_block)?;
                let then_cast = TypeConversion::from_target_type(result_type, self.state)
                    .emit(then_value, self.state, &then_end);
                result_slot.store(then_cast, self.state, &then_end);
                Effect::new(self.state, then_end).r#yield(&[]);

                let false_expression = conditional.false_expression();
                let (else_value, else_end) = self.emit_value(&false_expression, else_block)?;
                let else_cast = TypeConversion::from_target_type(result_type, self.state)
                    .emit(else_value, self.state, &else_end);
                result_slot.store(else_cast, self.state, &else_end);
                Effect::new(self.state, else_end).r#yield(&[]);

                let result = result_slot.load(result_type, self.state, &block);

                Ok((Some(result), block))
            }
            Expression::ArrayExpression(array_expression) => {
                let result_slang_type = array_expression
                    .get_type()
                    .expect("slang types every array literal");
                let element_slang_type = match &result_slang_type {
                    SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
                    SlangType::Array(array_type) => array_type.element_type(),
                    _ => anyhow::bail!(
                        "array literal has unexpected result type: {:?}",
                        std::mem::discriminant(&result_slang_type)
                    ),
                };
                let array_type =
                    TypeConversion::resolve_slang_type(&result_slang_type, None, self.state);
                let element_type =
                    TypeConversion::resolve_slang_type(&element_slang_type, None, self.state);
                let mut element_values = Vec::new();
                let mut current = block;
                for item in array_expression.items().iter() {
                    let (value, next) = self.emit_value(&item, current)?;
                    let cast_value = TypeConversion::from_target_type(element_type, self.state)
                        .emit(value, self.state, &next);
                    element_values.push(cast_value);
                    current = next;
                }
                let value = Value::array_literal(&element_values, array_type, self.state, &current);
                Ok((Some(value), current))
            }
            Expression::MemberAccessExpression(access) => {
                if let Some((value, block)) = self.emit_struct_field(access, block)? {
                    Ok((Some(value), block))
                } else {
                    self::call::CallEmitter::new(self)
                        .emit_member_access(access, block)
                        .map(|(value, block)| (Some(value), block))
                }
            }
            Expression::IndexAccessExpression(index_access) => {
                self.emit_index_access(index_access, block)
            }
            _ => anyhow::bail!(
                "unsupported expression: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits a `sol.cmp ne 0` producing `i1` from a value.
    ///
    /// Short-circuits when the value is already `i1`, avoiding the redundant
    /// `sol.cmp ne, %i1, %zero_i1 : i1` pattern.
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        if value.r#type().is_integer()
            && value.r#type().integer_bit_width() == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            return value;
        }
        let zero = Value::constant(0, value.r#type(), self.state, block);
        value.compare(zero, CmpPredicate::Ne, self.state, block)
    }

    /// Resolves the Solidity type from Slang to an MLIR type.
    ///
    /// Returns `None` when the incoming slang type is `None`. This can happen when calling
    /// `node.get_type()` if the node doesn't have typing information, for example when
    /// there are unresolved references or semantic errors.
    /// Panics on types that `TypeConversion::resolve_slang_type` does not yet handle.
    // TODO: slang's binder does not fold binary expressions of literal operands;
    // its typing rules return the type of one operand, so `1 << 100` gets typed
    // as ui8 and constant subexpressions overflow at that width. Either teach
    // slang to fold, or fold here before lowering.
    pub fn resolve_slang_type(&self, slang_type: Option<SlangType>) -> Option<Type<'context>> {
        Some(TypeConversion::resolve_slang_type(
            &slang_type?,
            None,
            self.state,
        ))
    }

    /// Picks the MLIR type of the address yielded by `sol.gep` / `sol.map`.
    ///
    /// Mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule: when the
    /// element is itself a reference type and lives in `Storage` or
    /// `CallData`, the result address IS the element type rather than a
    /// pointer to it.
    fn address_type(
        context: &Context<'context>,
        element_type: Type<'context>,
        base_location: DataLocation,
        result_type: &SlangType,
    ) -> Type<'context> {
        if result_type.is_reference_type()
            && matches!(
                base_location,
                DataLocation::Storage | DataLocation::CallData
            )
        {
            element_type
        } else {
            Type::pointer(context.melior, element_type, base_location)
        }
    }
}
