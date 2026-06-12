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
use melior::ir::r#type::IntegerType;
use operator::Operator;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::UserDefinedOperator;
use solx_mlir::ods::sol::ThisOperation;
use solx_utils::BIT_LENGTH_BYTE;
use solx_utils::DataLocation;

use crate::ast::ExpressionExt;
use crate::ast::LibraryExt;
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
            *value = TypeConversion::coerce(*value, parameter_type, &self.state.builder, block);
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

    /// Emits a value-position expression toward a known `target_type`,
    /// special-casing a string literal converted to a fixed-bytes type.
    ///
    /// A string literal used where `bytesN` / `byte` is expected (`bytes7 x =
    /// "abc"`, `b == "1234567"`, a `bytes`/`string` element `s[i] = "c"`) is a
    /// compile-time fixed-bytes constant, not a runtime `sol.string`: emitting
    /// the runtime string and letting the coercion `sol.cast` it fails the
    /// integer-only verifier (`operand #0 must be integer, but got !sol.string`).
    /// `bytesN` is left-aligned — the literal occupies the high-order bytes,
    /// zero-padded on the right. Every other expression / target combination is
    /// a plain [`Self::emit_value`], so routing a coercion site through this is a
    /// pure superset.
    pub fn emit_value_for_target(
        &self,
        expression: &Expression,
        target_type: Type<'context>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Expression::StringExpression(string_expression) = expression {
            let builder = &self.state.builder;
            // A string literal toward a single `byte` (an element of
            // `bytes`/`string`) materialises as a `!sol.byte` constant.
            if solx_mlir::TypeFactory::is_sol_byte(target_type) {
                let byte = string_expression.value().first().copied().unwrap_or(0);
                let ui8 = Type::from(IntegerType::unsigned(
                    builder.context,
                    BIT_LENGTH_BYTE as u32,
                ));
                let integer = builder.emit_constant(&num_bigint::BigInt::from(byte), ui8, &block);
                let value = builder.emit_sol_bytes_cast(integer, target_type, &block);
                return Ok((value, block));
            }
            if let Some(width) = solx_mlir::TypeFactory::fixed_bytes_or_byte_width(target_type) {
                let literal_bytes = string_expression.value();
                let mut buffer = vec![0u8; width as usize];
                for (slot, byte) in buffer.iter_mut().zip(literal_bytes.iter()) {
                    *slot = *byte;
                }
                let int_value = num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, &buffer);
                let integer_type = Type::from(IntegerType::unsigned(
                    builder.context,
                    width * BIT_LENGTH_BYTE as u32,
                ));
                let integer = builder.emit_constant(&int_value, integer_type, &block);
                let value =
                    builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(width), &block);
                return Ok((value, block));
            }
        }
        self.emit_value(expression, block)
    }

    /// Emits the two operands of a binary expression, materialising a string
    /// literal compared/combined with a `bytesN` / `byte` operand (`b == "d"`,
    /// `b | "x"`) as a fixed-bytes constant rather than a runtime `sol.string`.
    ///
    /// The non-string operand is emitted first to learn its MLIR type; the
    /// string operand is then emitted toward it via [`Self::emit_value_for_target`]
    /// when that type is fixed-bytes-like. When neither (or both) operand is a
    /// string literal, this is exactly two `emit_value` calls.
    pub fn emit_binary_operands(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let is_bytes_like =
            |ty: Type<'context>| solx_mlir::TypeFactory::fixed_bytes_or_byte_width(ty).is_some();
        let left_is_string = matches!(left, Expression::StringExpression(_));
        let right_is_string = matches!(right, Expression::StringExpression(_));
        if right_is_string && !left_is_string {
            let (lhs, block) = self.emit_value(left, block)?;
            let (rhs, block) = if is_bytes_like(lhs.r#type()) {
                self.emit_value_for_target(right, lhs.r#type(), block)?
            } else {
                self.emit_value(right, block)?
            };
            Ok((lhs, rhs, block))
        } else if left_is_string && !right_is_string {
            let (rhs, block) = self.emit_value(right, block)?;
            let (lhs, block) = if is_bytes_like(rhs.r#type()) {
                self.emit_value_for_target(left, rhs.r#type(), block)?
            } else {
                self.emit_value(left, block)?
            };
            Ok((lhs, rhs, block))
        } else {
            let (lhs, block) = self.emit_value(left, block)?;
            let (rhs, block) = self.emit_value(right, block)?;
            Ok((lhs, rhs, block))
        }
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
        // Constant folding: slang assigns a `Literal` type carrying the exact
        // computed value to compile-time-constant arithmetic/bitwise expressions.
        // Emitting that value directly matches solc's exact rational arithmetic
        // (`1/2*2 == 1`, `2**256-1` without 256-bit wraparound) and is the only
        // way to lower a rational intermediate, which has no runtime type.
        if Self::is_foldable_constant(expression) {
            let value = self.emit_folded_constant(expression, &block);
            return Ok((Some(value), block));
        }
        match expression {
            Expression::DecimalNumberExpression(decimal_number) => {
                let value = decimal_number
                    .integer_value()
                    .expect("a lowered decimal literal evaluates to an integer after units");
                let result_type = TypeConversion::resolve_optional_slang_type(
                    decimal_number.get_type(),
                    &self.state.builder,
                )
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
                let result_type = TypeConversion::resolve_optional_slang_type(
                    hex_number.get_type(),
                    &self.state.builder,
                )
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
                let value = sol_op!(
                    &self.state.builder,
                    block,
                    ThisOperation.addr(contract_type)
                );
                Ok((Some(value), block))
            }

            Expression::StringExpression(string_expression) => {
                // A string literal's bytes are emitted verbatim — they need not be
                // valid UTF-8 (`hex"..."`, `"\xff"`).
                let bytes = string_expression.value();
                let value = self.state.builder.emit_sol_string_lit_bytes(&bytes, &block);
                Ok((Some(value), block))
            }
            Expression::Identifier(identifier) => match identifier.resolve_to_definition() {
                Some(Definition::StateVariable(state_variable)) => self
                    .emit_state_variable_read(&state_variable, block)
                    .map(|(value, block)| (Some(value), block)),
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
                Some(Definition::Function(function_definition)) => self
                    .emit_internal_function_pointer(&function_definition, block)
                    .map(|(value, block)| (Some(value), block)),
                Some(Definition::Library(library)) => {
                    // A library name used as a value (`address(L)`) is its linked
                    // deploy address, placed by its link symbol.
                    let value = self
                        .state
                        .builder
                        .emit_sol_lib_addr(&library.link_symbol(), &block);
                    Ok((Some(value), block))
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
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
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
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
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
                let target_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
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
                // Peel parenthesised single-element tuples so `(i)++` / `(arr[j])--`
                // resolve their lvalue exactly like the bare `i++` / `arr[j]--`.
                let operand = expression.operand().unwrap_parens();
                let operator = match expression.operator() {
                    ast::PostfixExpressionOperator::MinusMinus(_) => Operator::Decrement,
                    ast::PostfixExpressionOperator::PlusPlus(_) => Operator::Increment,
                };
                self.emit_postfix(&operand, operator, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::PrefixExpression(expression) => {
                // `delete x` zeroes an lvalue and is void-typed, so it is routed
                // before the result-type resolution the arithmetic/logical prefix
                // operators need (resolving a void type would fail).
                if let ast::PrefixExpressionOperator::DeleteKeyword(_) = expression.operator() {
                    return self
                        .emit_delete(&expression.operand(), block)
                        .map(|block| (None, block));
                }
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
                let operator = match expression.operator() {
                    ast::PrefixExpressionOperator::Bang(_) => Operator::Not,
                    ast::PrefixExpressionOperator::DeleteKeyword(_) => {
                        unreachable!("delete is routed before prefix-operator classification")
                    }
                    ast::PrefixExpressionOperator::Minus(_) => Operator::Subtract,
                    ast::PrefixExpressionOperator::MinusMinus(_) => Operator::Decrement,
                    ast::PrefixExpressionOperator::PlusPlus(_) => Operator::Increment,
                    ast::PrefixExpressionOperator::Tilde(_) => Operator::BitwiseNot,
                };
                // Peel parenthesised single-element tuples so `--(i)` / `~(x)`
                // operate on the bare inner lvalue / value, as solc treats them.
                let operand = expression.operand().unwrap_parens();
                self.emit_prefix(operator, &operand, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseAndExpression(expression) => {
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::BitwiseAnd, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseOrExpression(expression) => {
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::BitwiseOr, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::BitwiseXorExpression(expression) => {
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
                let left = expression.left_operand();
                let right = expression.right_operand();
                self.emit_binary_op(&left, &right, Operator::BitwiseXor, result_type, block)
                    .map(|(value, block)| (Some(value), block))
            }
            Expression::ShiftExpression(expression) => {
                let result_type = TypeConversion::resolve_optional_slang_type(
                    expression.get_type(),
                    &self.state.builder,
                );
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
                // A namespace-qualified state-variable / constant read — `C.x`,
                // `L.CONST`, `M.a` — reads the named member exactly like the bare
                // identifier would, disambiguating from a shadowing local. The
                // operand must be a namespace name (a contract / library / import
                // alias); `this.x` keeps the external-getter path since its
                // operand is the `this` keyword, not an identifier.
                if let Expression::Identifier(operand) = access.operand()
                    && matches!(
                        operand.resolve_to_definition(),
                        Some(
                            Definition::Contract(_)
                                | Definition::Library(_)
                                | Definition::Import(_)
                                | Definition::ImportedSymbol(_)
                        )
                    )
                {
                    match access.member().resolve_to_definition() {
                        Some(Definition::StateVariable(state_variable)) => {
                            return self
                                .emit_state_variable_read(&state_variable, block)
                                .map(|(value, block)| (Some(value), block));
                        }
                        Some(Definition::Constant(constant)) => {
                            let initializer = constant
                                .value()
                                .expect("a Solidity constant has an initializer");
                            return self.emit(&initializer, block);
                        }
                        _ => {}
                    }
                }
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
            Expression::CallOptionsExpression(call_options) => {
                // A call-options expression in value position (decorated but not
                // immediately called) contributes only its options' side effects;
                // its value is that of the wrapped operand.
                let mut current_block = block;
                for option in call_options.options().iter() {
                    let (_value, next) = self.emit_value(&option.value(), current_block)?;
                    current_block = next;
                }
                self.emit(&call_options.operand(), current_block)
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

    /// Whether `expression` is a compile-time-constant arithmetic/bitwise
    /// expression that slang has folded to an exact integer — the case
    /// [`Self::emit`] lowers straight to a constant instead of runtime ops.
    /// Only computed expressions qualify (a bare literal keeps its own arm);
    /// a non-integer rational is excluded, having no integer constant to emit.
    fn is_foldable_constant(expression: &Expression) -> bool {
        use slang_solidity_v2::ast::LiteralKind;
        let is_computed = matches!(
            expression,
            Expression::AdditiveExpression(_)
                | Expression::MultiplicativeExpression(_)
                | Expression::ExponentiationExpression(_)
                | Expression::ShiftExpression(_)
                | Expression::BitwiseAndExpression(_)
                | Expression::BitwiseOrExpression(_)
                | Expression::BitwiseXorExpression(_)
                | Expression::PrefixExpression(_)
        );
        if !is_computed {
            return false;
        }
        let Some(SlangType::Literal(literal_type)) = expression.get_type() else {
            return false;
        };
        match literal_type.kind() {
            LiteralKind::Integer { .. } | LiteralKind::HexInteger { .. } => true,
            LiteralKind::Rational { value } => value.is_integer(),
            _ => false,
        }
    }

    /// Emits the folded integer value of a constant expression that
    /// [`Self::is_foldable_constant`] has accepted (its invariants are relied on
    /// here: a `Literal` type whose kind is an integer or integer-valued rational).
    fn emit_folded_constant(
        &self,
        expression: &Expression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        use slang_solidity_v2::ast::LiteralKind;
        let Some(SlangType::Literal(literal_type)) = expression.get_type() else {
            unreachable!("is_foldable_constant guarantees a literal type");
        };
        let value = match literal_type.kind() {
            LiteralKind::Integer { value } => value,
            // A hex literal's value is now an unsigned `BigUint`; widen to the
            // signed `BigInt` the constant emitter expects.
            LiteralKind::HexInteger { value, .. } => num_bigint::BigInt::from(value),
            LiteralKind::Rational { value } => value.to_integer(),
            _ => unreachable!("is_foldable_constant guarantees a numeric literal"),
        };
        let result_type =
            TypeConversion::resolve_optional_slang_type(expression.get_type(), &self.state.builder)
                .expect("binder types every folded constant expression");
        self.state.builder.emit_constant(&value, result_type, block)
    }

    /// Emits an internal function used as a value (`g` in `f = g;`) as a
    /// `sol.func_constant` producing an `!sol.func_ref<…>` pointer.
    ///
    /// The target is routed through the virtual redirect exactly as a direct
    /// call is, so a base-body `f = g` binds the most-derived override of `g`
    /// — the lexical base version is shadowed and thus unregistered when the
    /// derived contract is compiled.
    fn emit_internal_function_pointer(
        &self,
        function_definition: &FunctionDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let node_id = function_definition.node_id();
        let target_id = self
            .state
            .virtual_redirect
            .get(&node_id)
            .copied()
            .unwrap_or(node_id);
        self.emit_function_constant(target_id, block)
    }

    /// Emits a `sol.func_constant` for the already-resolved internal function
    /// `target_id`, producing its `!sol.func_ref<…>` pointer. The literal target
    /// lowers as-is (no virtual redirect); a caller wanting the most-derived
    /// override resolves the redirect first (see
    /// [`Self::emit_internal_function_pointer`]).
    pub fn emit_function_constant(
        &self,
        target_id: NodeId,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (mlir_name, parameter_types, return_types) = self.state.resolve_function(target_id)?;
        let func_ref_type = self
            .state
            .builder
            .types
            .func_ref(parameter_types, return_types);
        let mlir_name = mlir_name.to_owned();
        let value = self
            .state
            .builder
            .emit_sol_func_constant(&mlir_name, func_ref_type, &block);
        Ok((value, block))
    }

    /// If `expression` is a bare function name — always an *internal* function
    /// pointer — returns its `!sol.func_ref` type, built from the function's
    /// declared signature. slang types such a reference from the function's
    /// visibility (a `Public` function resolves to its return type, not the
    /// pointer type), so a caller inferring a result type from the expression —
    /// e.g. a ternary whose branches are function names — uses this to recover
    /// the authoritative internal-pointer type the branch values carry. Returns
    /// `None` for any expression that is not a bare reference to a function.
    fn bare_function_ref_type(&self, expression: &Expression) -> Option<Type<'context>> {
        let Expression::Identifier(identifier) = expression else {
            return None;
        };
        let Some(Definition::Function(function_definition)) = identifier.resolve_to_definition()
        else {
            return None;
        };
        let (_, parameter_types, return_types) = self
            .state
            .resolve_function(function_definition.node_id())
            .ok()?;
        Some(
            self.state
                .builder
                .types
                .func_ref(parameter_types, return_types),
        )
    }

    /// Reads a contract state variable's value: a `constant` inlines its
    /// compile-time initializer (exactly as a file-level `constant`), otherwise
    /// the storage slot is loaded. A value-typed slot reads through the shared
    /// storage-load helper; a reference-typed one evaluates to its storage
    /// reference, whose address type is the reference itself (the single
    /// `address_type` rule). Shared by a bare identifier reference and a
    /// namespace-qualified `C.stateVar` / `L.CONST` access (the latter
    /// disambiguating from a shadowing local).
    fn emit_state_variable_read(
        &self,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let declared_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        if matches!(
            state_variable.mutability(),
            StateVariableMutability::Constant
        ) {
            let initializer = state_variable
                .value()
                .expect("a constant state variable has an initializer");
            // Emit toward the declared type so a `bytesN constant` initialised
            // from a string literal folds to a fixed-bytes constant.
            return self.emit_value_for_target(&initializer, element_type, block);
        }
        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unimplemented!("unregistered state variable {:?}", state_variable.node_id())
            });
        let value = if declared_type.is_reference_type() {
            let address_type = Self::address_type(
                &self.state.builder,
                element_type,
                slot.location,
                &declared_type,
            );
            let address = self
                .state
                .builder
                .emit_sol_addr_of(&slot.name, address_type, &block);
            self.state
                .builder
                .emit_sol_load(address, element_type, &block)?
        } else {
            self.emit_storage_load(slot, element_type, &block)?
        };
        Ok((value, block))
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
