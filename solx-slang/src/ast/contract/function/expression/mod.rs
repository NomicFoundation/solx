//!
//! Expression lowering to MLIR SSA values.
//!

pub mod arithmetic;
pub mod arithmetic_mode;
pub mod assignment;
pub mod call;
pub mod comparison;
pub mod conditional;
pub mod index_access;
pub mod member;
pub mod new;
pub mod operator;
pub mod short_circuit;
pub mod storage;
pub mod unary;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use operator::Operator;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::UserDefinedOperator;
use solx_mlir::ods::sol::ThisOperation;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::TypeConversion;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment.
    environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Arithmetic overflow-checking mode for binary operations.
    ///
    /// [`ArithmeticMode::Checked`] by default (Solidity 0.8+);
    /// [`ArithmeticMode::Unchecked`] inside `unchecked {}` blocks and for-loop
    /// step expressions.
    arithmetic_mode: ArithmeticMode,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        arithmetic_mode: ArithmeticMode,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            arithmetic_mode,
        }
    }

    /// The function bound to `user_operator` for `operand`'s user-defined value
    /// type via `using {f as op} for T global;`, or `None` when `operand` is not
    /// such a type or the operator carries no binding. The shared UDVT lookup
    /// behind the binary ([`Self::user_defined_binary_operator`]) and unary
    /// ([`Self::user_defined_unary_operator`]) operator classification.
    ///
    /// [`Self::user_defined_binary_operator`]: ExpressionEmitter::user_defined_binary_operator
    /// [`Self::user_defined_unary_operator`]: ExpressionEmitter::user_defined_unary_operator
    fn user_defined_operator(
        &self,
        operand: &Expression,
        user_operator: UserDefinedOperator,
    ) -> Option<NodeId> {
        let SlangType::UserDefinedValue(udvt_type) = operand.get_type()? else {
            return None;
        };
        let Definition::UserDefinedValueType(udvt_definition) = udvt_type.definition() else {
            return None;
        };
        self.state
            .operator_bindings
            .get(&(udvt_definition.node_id(), user_operator))
            .copied()
    }

    /// Calls the bound user-defined-operator function `function_id` with the
    /// already-evaluated `argument_values`, each coerced to its parameter type,
    /// and returns the operator's single result value. Shared by the binary
    /// ([`Self::emit_binary_op`]) and unary ([`Self::emit_prefix`]) operator
    /// dispatch (`using {f as op} for T global;`).
    fn emit_operator_call(
        &self,
        function_id: NodeId,
        mut argument_values: Vec<Value<'context, 'block>>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let (mlir_name, parameter_types, return_types) =
            self.state.resolve_function(function_id)?;
        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            *value = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                *value,
                &self.state.builder,
                block,
            );
        }
        let results = self.state.builder.emit_sol_call_results(
            mlir_name,
            &argument_values,
            return_types,
            block,
        )?;
        Ok(results
            .into_iter()
            .next()
            .expect("a user-defined operator returns one value"))
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
        let value = value.expect("an expression in value position produces a value");
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
            Expression::DecimalNumberExpression(decimal_number) => {
                let value = decimal_number
                    .integer_value()
                    .expect("a lowered decimal literal evaluates to an integer after units");
                let result_type = self
                    .resolve_slang_type(decimal_number.get_type())
                    .expect("binder types every decimal literal node");
                let constant = self
                    .state
                    .builder
                    .emit_constant(&value, result_type, &block);
                Ok((Some(constant), block))
            }
            Expression::HexNumberExpression(hex_number) => {
                let value = hex_number
                    .integer_value()
                    .expect("hex literals always evaluate to integers");
                let result_type = self
                    .resolve_slang_type(hex_number.get_type())
                    .expect("binder types every hex literal node");
                let constant = self
                    .state
                    .builder
                    .emit_constant(&value, result_type, &block);
                Ok((Some(constant), block))
            }
            Expression::TrueKeyword(_) => {
                let constant = self.state.builder.emit_bool(true, &block);
                Ok((Some(constant), block))
            }
            Expression::FalseKeyword(_) => {
                let constant = self.state.builder.emit_bool(false, &block);
                Ok((Some(constant), block))
            }
            Expression::ThisKeyword(_) => {
                let contract_type = self
                    .state
                    .current_contract_type
                    .expect("`this` only appears inside a contract method");
                let operation = ThisOperation::builder(
                    self.state.builder.context,
                    self.state.builder.unknown_location,
                )
                .addr(contract_type)
                .build();
                let value = block
                    .append_operation(operation.into())
                    .result(0)
                    .expect("sol.this always produces one result")
                    .into();
                Ok((Some(value), block))
            }
            Expression::StringExpression(string_expression) => {
                let bytes = string_expression.value();
                let text = std::str::from_utf8(&bytes).expect("string literal is valid UTF-8");
                let value = self.state.builder.emit_sol_string_lit(text, &block);
                Ok((Some(value), block))
            }
            Expression::Identifier(identifier) => match identifier.resolve_to_definition() {
                Some(Definition::StateVariable(state_variable)) => {
                    let slot = self
                        .storage_layout
                        .get(&state_variable.node_id())
                        .unwrap_or_else(|| {
                            unimplemented!(
                                "unregistered state variable {:?}",
                                state_variable.node_id()
                            )
                        });
                    let declared_type = state_variable
                        .get_type()
                        .expect("slang types every state variable");
                    let element_type = TypeConversion::resolve_slang_type(
                        &declared_type,
                        None,
                        &self.state.builder,
                    );
                    // A value-typed state variable reads through the shared
                    // storage-load helper. A reference-typed one evaluates to its
                    // storage reference, whose address type is the reference
                    // itself — the single `address_type` rule, shared with the
                    // initializer path.
                    let value = if declared_type.is_reference_type() {
                        let address_type = Self::address_type(
                            &self.state.builder,
                            element_type,
                            DataLocation::Storage,
                            &declared_type,
                        );
                        let address =
                            self.state
                                .builder
                                .emit_sol_addr_of(&slot.name, address_type, &block);
                        self.state
                            .builder
                            .emit_sol_load(address, element_type, &block)?
                    } else {
                        self.emit_storage_load(slot, element_type, &block)?
                    };
                    Ok((Some(value), block))
                }
                Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                    let (pointer, element_type) =
                        self.environment.variable_with_type(definition.node_id());
                    let value = self
                        .state
                        .builder
                        .emit_sol_load(pointer, element_type, &block)?;
                    Ok((Some(value), block))
                }
                Some(Definition::Constant(constant)) => {
                    let initializer = constant
                        .value()
                        .expect("a Solidity constant has an initializer");
                    self.emit(&initializer, block)
                }
                None => unreachable!("slang resolves every identifier reference"),
                Some(other) => {
                    unimplemented!("unsupported identifier reference {:?}", other.node_id())
                }
            },
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
                    // `delete x` zeroes an lvalue — not an arithmetic/logical
                    // prefix operator — so it is its own (loud) residual rather
                    // than an `Operator` variant the prefix emitter must reject.
                    ast::PrefixExpressionOperator::DeleteKeyword(_) => {
                        unimplemented!("delete is not yet supported")
                    }
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
            Expression::TupleExpression(tuple) => self.emit_tuple(tuple, block),
            Expression::ConditionalExpression(conditional) => self
                .emit_conditional(conditional, block)
                .map(|(value, block)| (Some(value), block)),
            Expression::ArrayExpression(array_expression) => self
                .emit_array_literal(array_expression, block)
                .map(|(value, block)| (Some(value), block)),
            Expression::MemberAccessExpression(access) => {
                // A struct-typed base is a field read (`s.field`); anything else
                // (e.g. `msg.sender`, `addr.balance`) is a built-in member access.
                if matches!(access.operand().get_type(), Some(SlangType::Struct(_))) {
                    self.emit_struct_field(access, block)
                        .map(|(value, block)| (Some(value), block))
                } else {
                    self::call::CallEmitter::new(self)
                        .emit_member_access(access, block)
                        .map(|(value, block)| (Some(value), block))
                }
            }
            Expression::IndexAccessExpression(index_access) => {
                self.emit_index_access(index_access, block)
            }
            Expression::CallOptionsExpression(_) => {
                unimplemented!("expression lowering: call options")
            }
            Expression::NewExpression(_)
            | Expression::TypeExpression(_)
            | Expression::ElementaryType(_)
            | Expression::PayableKeyword(_)
            | Expression::SuperKeyword(_) => {
                unimplemented!("expression lowering: bare type/keyword")
            }
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

    /// Resolves the Solidity type from Slang to an MLIR type.
    ///
    /// Returns `None` when the incoming slang type is `None`. This can happen when calling
    /// `node.get_type()` if the node doesn't have typing information, for example when
    /// there are unresolved references or semantic errors.
    /// Panics on types that `TypeConversion::resolve_slang_type` does not yet handle.
    // TODO: slang's binder does not fold binary expressions of literal operands —
    // its typing rules return the type of one operand (e.g. type of the left
    // operand for shifts), so `1 << 100` gets typed as ui8 (the type of `1`)
    // and constant subexpressions overflow at that width. solc folds via
    // `RationalNumberType::binaryOperatorResult`, sizing the result to fit the
    // folded value. Either teach slang to fold, or fold here before lowering.
    pub fn resolve_slang_type(&self, slang_type: Option<SlangType>) -> Option<Type<'context>> {
        Some(TypeConversion::resolve_slang_type(
            &slang_type?,
            None,
            &self.state.builder,
        ))
    }

    /// Picks the MLIR type of the address yielded by `sol.gep` / `sol.map`.
    ///
    /// Mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule: when the
    /// element is itself a reference type and lives in `Storage` or
    /// `CallData`, the result address IS the element type rather than a
    /// pointer to it.
    fn address_type(
        builder: &Builder<'context>,
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
            builder.types.pointer(element_type, base_location)
        }
    }
}
