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

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use operator::Operator;
use ruint::aliases::U256;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::ThisOperation;
use solx_utils::DataLocation;

use self::call::type_conversion::TypeConversion;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub(crate) state: &'state Context<'context>,
    /// Variable environment.
    pub(crate) environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub(crate) storage_layout: &'state HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>,
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default (Solidity 0.8+). Set to `false` inside `unchecked {}`
    /// blocks and for-loop step expressions.
    pub(crate) checked: bool,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, (U256, u32, solx_utils::DataLocation)>,
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
    /// Delegates to [`Self::emit`] and returns an error for void expressions
    /// (e.g. calls to functions with no return value).
    pub fn emit_value(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(expression, block)?;
        let value = value.expect("expression produced no value");
        Ok((value, block))
    }

    /// Emits an expression toward a known target MLIR type. The only special
    /// case is a string literal used where a `bytesN` value is expected
    /// (`bytes2 a = "a"`, a `bytes3` argument, `return "abc"`): slang types the
    /// literal as a string, so a plain emit + `bytes_cast` would feed a memory
    /// string into the integer-only cast. Emit a left-aligned fixedbytes
    /// constant directly instead. Every other expression defers to
    /// [`Self::emit_value`] (the caller still applies its own coercion).
    pub fn emit_value_for_target(
        &self,
        expression: &Expression,
        target_type: Type<'context>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if let Expression::StringExpression(string_expression) = expression
            && let Some(width) = solx_mlir::TypeFactory::fixed_bytes_width(target_type)
        {
            let literal_bytes = string_expression.value();
            // Fixedbytes are left-aligned: the literal occupies the high-order
            // bytes, zero-padded on the right.
            let mut buffer = vec![0u8; width as usize];
            for (slot, byte) in buffer.iter_mut().zip(literal_bytes.iter()) {
                *slot = *byte;
            }
            let int_value = num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, &buffer);
            let integer_type = Type::from(melior::ir::r#type::IntegerType::unsigned(
                self.state.builder.context,
                width * solx_utils::BIT_LENGTH_BYTE as u32,
            ));
            let integer = self
                .state
                .builder
                .emit_constant(&int_value, integer_type, &block);
            let value = self.state.builder.emit_sol_bytes_cast(
                integer,
                self.state.builder.types.fixed_bytes(width),
                &block,
            );
            return Ok((value, block));
        }
        // A string literal assigned to a single byte — the element of
        // `bytes`/`string`, `x[i] = "c"` — materializes as a `!sol.byte`
        // constant. `sol.bytes_cast` rejects a dynamic-string operand, so a
        // plain `emit_value` (which emits `sol.string_lit`) would fail to cast.
        if let Expression::StringExpression(string_expression) = expression
            && solx_mlir::TypeFactory::is_sol_byte(target_type)
        {
            let literal_bytes = string_expression.value();
            let byte = literal_bytes.first().copied().unwrap_or(0);
            let ui8 = Type::from(melior::ir::r#type::IntegerType::unsigned(
                self.state.builder.context,
                solx_utils::BIT_LENGTH_BYTE as u32,
            ));
            let integer = self
                .state
                .builder
                .emit_constant(&num_bigint::BigInt::from(byte), ui8, &block);
            let value = self
                .state
                .builder
                .emit_sol_bytes_cast(integer, target_type, &block);
            return Ok((value, block));
        }
        self.emit_value(expression, block)
    }

    /// Emits the two operands of a binary expression (comparison, arithmetic,
    /// bitwise), materializing a string literal paired with a `bytesN` / `byte`
    /// operand (`b == "d"`, `b | "a"`) as a fixedbytes/byte constant rather than
    /// a memory string — which a plain emit would feed into the integer-only
    /// operand cast. String literals are side-effect-free, so the literal is
    /// emitted after the other operand (once its type is known), preserving the
    /// observable left-to-right evaluation order of any side effects.
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
        let is_bytes_like = |ty: Type<'context>| {
            solx_mlir::TypeFactory::is_sol_fixed_bytes(ty)
                || solx_mlir::TypeFactory::is_sol_byte(ty)
        };
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
    // A flat per-expression dispatch: one `match expression` arm per
    // [`Expression`] variant, most delegating to a dedicated `emit_*` method.
    // The length is inherent to the variant count, not nesting; the one
    // genuinely-nested sub-algorithm (an internal function-pointer value) is
    // factored into `emit_internal_function_pointer`.
    #[allow(clippy::too_many_lines)]
    pub fn emit(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // Constant folding: slang assigns a `Literal` type carrying the
        // computed value to compile-time constant (sub)expressions. Emitting
        // that value directly matches solc's exact rational arithmetic — e.g.
        // `1/2*2 == 1` and `2**256-1` (which would overflow if evaluated with
        // runtime 256-bit ops). Only computed expressions are folded; simple
        // literals/identifiers keep their existing paths.
        if Self::is_foldable_expression(expression)
            && let Some(value) = self.try_emit_constant_literal(expression, &block)
        {
            return Ok((Some(value), block));
        }
        match expression {
            Expression::DecimalNumberExpression(decimal_number) => {
                let value = decimal_number.integer_value().expect("decimal literal cannot be lowered: it must evaluate to an integer \
                         after applying any units");
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
                    .expect("sol.this emitted outside a contract");
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
                // `hex"…"`/escaped string literals decode to arbitrary bytes
                // that need not be valid UTF-8 (e.g. `hex"…9A"`); emit them
                // verbatim rather than round-tripping through a `&str`, which
                // re-encodes bytes ≥ 0x80 as multi-byte UTF-8 and corrupts them.
                let bytes = string_expression.value();
                let value = self.state.builder.emit_sol_string_lit_bytes(&bytes, &block);
                Ok((Some(value), block))
            }
            Expression::Identifier(identifier) => {
                let name = identifier.name();
                match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => self
                        .emit_state_variable_read(&state_variable, block)
                        .map(|(value, block)| (Some(value), block)),
                    Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                        let (pointer, element_type) = self.environment.variable_with_type(&name);
                        let value =
                            self.state
                                .builder
                                .emit_sol_load(pointer, element_type, &block)?;
                        Ok((Some(value), block))
                    }
                    Some(Definition::Constant(constant)) => {
                        let initializer = constant
                            .value()
                            .expect("constant has no initializer");
                        self.emit(&initializer, block)
                    }
                    Some(Definition::Function(function_definition)) => self
                        .emit_internal_function_pointer(&function_definition, block)
                        .map(|(value, block)| (Some(value), block)),
                    Some(Definition::Library(library)) => {
                        // A library name used as a value (`address(L)`) is its
                        // linked deploy address. The linker symbol is the
                        // fully-qualified `file:Library` name (matching solc),
                        // so `link_references` round-trips.
                        let symbol =
                            format!("{}:{}", library.get_file_id(), library.name().name());
                        let value = self.state.builder.emit_sol_lib_addr(&symbol, &block);
                        Ok((Some(value), block))
                    }
                    None => unreachable!("unresolved identifier: {name}"),
                    Some(_) => unimplemented!("unsupported identifier reference: {name}"),
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
                if items.len() != 1 {
                    unimplemented!("multi-value tuple expression");
                }
                let item = items.iter().next().expect("length checked to be 1 above");
                let inner = item
                    .expression()
                    .expect("empty tuple element");
                self.emit(&inner, block)
            }
            Expression::ConditionalExpression(conditional) => {
                let resolved_type = self
                    .resolve_slang_type(conditional.get_type())
                    .unwrap_or(self.state.builder.types.ui256);
                // A ternary whose branches are bare function names yields an
                // *internal* function pointer, but slang types it from the
                // functions' `Public` visibility (resolved to `ext_func_ref`).
                // The branch values are emitted as `func_ref`, so recover the
                // internal-pointer result type to match.
                let result_type = if solx_mlir::TypeFactory::is_sol_function_ref(resolved_type) {
                    self.bare_function_ref_type(&conditional.true_expression())
                        .or_else(|| {
                            self.bare_function_ref_type(&conditional.false_expression())
                        })
                        .unwrap_or(resolved_type)
                } else {
                    resolved_type
                };
                let condition = conditional.operand();
                let (condition_value, block) = self.emit_value(&condition, block)?;
                let condition_boolean = self.emit_is_nonzero(condition_value, &block);

                let result_slot = self.state.builder.emit_sol_alloca(result_type, &block);
                let (then_block, else_block) =
                    self.state.builder.emit_sol_if(condition_boolean, &block);

                let true_expression = conditional.true_expression();
                let (then_value, then_end) = self.emit_value(&true_expression, then_block)?;
                let then_cast = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(then_value, &self.state.builder, &then_end);
                self.state
                    .builder
                    .emit_sol_store(then_cast, result_slot, &then_end);
                self.state.builder.emit_sol_yield(&then_end);

                let false_expression = conditional.false_expression();
                let (else_value, else_end) = self.emit_value(&false_expression, else_block)?;
                let else_cast = TypeConversion::from_target_type(result_type, &self.state.builder)
                    .emit(else_value, &self.state.builder, &else_end);
                self.state
                    .builder
                    .emit_sol_store(else_cast, result_slot, &else_end);
                self.state.builder.emit_sol_yield(&else_end);

                let result = self
                    .state
                    .builder
                    .emit_sol_load(result_slot, result_type, &block)?;

                Ok((Some(result), block))
            }
            Expression::ArrayExpression(array_expression) => {
                let result_slang_type = array_expression
                    .get_type()
                    .expect("slang types every array literal");
                let element_slang_type = match &result_slang_type {
                    SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
                    SlangType::Array(array_type) => array_type.element_type(),
                    _ => unimplemented!(
                        "array literal has unexpected result type: {:?}",
                        std::mem::discriminant(&result_slang_type)
                    ),
                };
                let builder = &self.state.builder;
                let declared_element_type =
                    TypeConversion::resolve_slang_type(&element_slang_type, None, builder);
                // Emit the element values first. For a function-pointer array
                // literal the emitted values are authoritative: a bare function
                // name is an internal `func_ref`, but slang types the literal
                // from the function's `Public` visibility, which resolves to
                // `ext_func_ref`. Adopt the value's function-ref type when it
                // disagrees, and rebuild the array type to match.
                let mut element_values = Vec::new();
                let mut current = block;
                for item in array_expression.items().iter() {
                    let (value, next) = self.emit_value(&item, current)?;
                    element_values.push(value);
                    current = next;
                }
                let element_type = match element_values.first() {
                    Some(first)
                        if solx_mlir::TypeFactory::is_sol_function_ref(first.r#type())
                            && first.r#type() != declared_element_type =>
                    {
                        first.r#type()
                    }
                    _ => declared_element_type,
                };
                let array_type = if element_type == declared_element_type {
                    TypeConversion::resolve_slang_type(&result_slang_type, None, builder)
                } else if let SlangType::FixedSizeArray(fixed_array_type) = &result_slang_type {
                    builder.types.array(
                        solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                        element_type,
                        DataLocation::from_slang(fixed_array_type.location(), None),
                    )
                } else {
                    TypeConversion::resolve_slang_type(&result_slang_type, None, builder)
                };
                let element_values: Vec<_> = element_values
                    .into_iter()
                    .map(|value| {
                        TypeConversion::from_target_type(element_type, builder)
                            .emit(value, builder, &current)
                    })
                    .collect();
                let value = builder.emit_sol_array_lit(&element_values, array_type, &current);
                Ok((Some(value), current))
            }
            Expression::MemberAccessExpression(access) => {
                // A qualified state-variable / constant read whose operand is a
                // *namespace* rather than a value:
                //   - `C.stateVar` / `Base.stateVar` — contract-qualified (also
                //     reaches an inherited `constant`, disambiguates a shadowing
                //     local);
                //   - `L.CONST` — library-qualified constant;
                //   - `M.a` (`import "s1.sol" as M`) — an import-namespace-qualified
                //     file-level constant.
                // `this.stateVar` (an external getter call) keeps its own path
                // since its operand is the `this` keyword, not a namespace name.
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
                        // An inherited `constant` accessed as `A.x` resolves to a
                        // `Constant`; emit its initializer (foldable ones are
                        // already inlined by the constant-folding path).
                        Some(Definition::Constant(constant)) => {
                            let initializer = constant.value().expect("constant has no initializer");
                            return self.emit(&initializer, block);
                        }
                        _ => {}
                    }
                }
                // `(...).T` where the member resolves to nothing and the access
                // has no value type — a discarded type/namespace member reference
                // (e.g. the statement `(cond ? M : M).D;`, where slang can't type
                // a namespace-valued conditional, so `.D` does not resolve). It
                // has no runtime value: evaluate the operand for its side effects
                // and yield none. A value context would surface as
                // "expression produced no value" rather than be silently dropped.
                if access.member().resolve_to_definition().is_none()
                    && access.member().resolve_to_built_in().is_none()
                    && access.get_type().is_none()
                {
                    let block =
                        self.emit_discarded_operand_side_effects(&access.operand(), block)?;
                    return Ok((None, block));
                }
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
            Expression::CallOptionsExpression(call_options) => {
                // A function pointer carrying call options, used as a *value*
                // rather than being called here — e.g. `this.g{gas: 42}.address`
                // / `.selector`, where the options don't affect the address or
                // selector. The options are call-time parameters; as a value the
                // expression is the underlying function pointer. Evaluate each
                // option for its side effects, then yield the operand. (A genuine
                // call `fp{value: v}()` is handled in the call dispatch, which
                // threads `value` into the call instead of discarding it.)
                let mut current = block;
                for option in call_options.options().iter() {
                    let (_value, next) = self.emit_value(&option.value(), current)?;
                    current = next;
                }
                self.emit(&call_options.operand(), current)
            }
            // Unsupported lowering is a frontend capability gap, not a program
            // error — mark it with `unimplemented!` rather than the error
            // channel. (Speculative library emission sandboxes this panic; in a
            // contract it surfaces as INVALID, exactly as the old `bail!` did.)
            _ => unimplemented!(
                "expression lowering: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits a bare function name used as a value: always an *internal*
    /// function pointer (`sol.func_constant`) — an external pointer requires
    /// `this.f`. The `!sol.func_ref` type is built from the declared signature
    /// rather than the identifier's slang type, which reports `ext_func_ref`
    /// for a public function (visibility `Public`) and would mismatch the
    /// internal-pointer target.
    ///
    /// A pointer to a `virtual` function binds to the most-derived override
    /// (`ptr = g` in a base body, with the deployed contract overriding `g`),
    /// so the virtual redirect is applied to the target node exactly as a call
    /// does — the lexical base version is shadowed and thus unregistered when
    /// compiling the derived contract.
    fn emit_internal_function_pointer(
        &self,
        function_definition: &slang_solidity_v2::ast::FunctionDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let node_id = function_definition.node_id();
        let target_id = self
            .state
            .virtual_redirect
            .get(&node_id)
            .copied()
            .unwrap_or(node_id);
        let (mlir_name, parameter_types, return_types) = self
            .state
            .resolve_function(target_id)
            .expect("unregistered function pointer");
        let func_ref_type = self
            .state
            .builder
            .types
            .func_ref(parameter_types, return_types);
        let mlir_name = mlir_name.to_owned();
        let value =
            self.state
                .builder
                .emit_sol_func_constant(&mlir_name, func_ref_type, &block);
        Ok((value, block))
    }

    /// Emits the side effects of the operand of a *discarded* type/namespace
    /// member reference (`(cond ? M : M).D;`). A value-typed operand is emitted
    /// and its result discarded; a namespace/type operand carries no runtime
    /// value, so only an embedded side effect (e.g. an assignment in a `?:`
    /// condition) is observable — recurse through parentheses and emit a
    /// conditional's condition.
    fn emit_discarded_operand_side_effects(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        if expression.get_type().is_some() {
            let (_value, block) = self.emit_value(expression, block)?;
            return Ok(block);
        }
        match expression {
            Expression::TupleExpression(tuple) if tuple.items().len() == 1 => {
                match tuple.items().iter().next().and_then(|item| item.expression()) {
                    Some(inner) => self.emit_discarded_operand_side_effects(&inner, block),
                    None => Ok(block),
                }
            }
            Expression::ConditionalExpression(conditional) => {
                let (_value, block) = self.emit_value(&conditional.operand(), block)?;
                Ok(block)
            }
            _ => Ok(block),
        }
    }

    /// Reads a contract state variable's value: a `constant` folds to its
    /// initializer (emitted toward the declared type, so a `bytes32 constant`
    /// string literal becomes a fixedbytes constant), otherwise the storage slot
    /// is loaded. Shared by a bare identifier reference and a contract-qualified
    /// `C.stateVar` access (the latter disambiguates from a shadowing local).
    fn emit_state_variable_read(
        &self,
        state_variable: &slang_solidity_v2::ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let declared_type = state_variable
            .get_type()
            .expect("unresolved type for state variable");
        if matches!(
            state_variable.mutability(),
            StateVariableMutability::Constant
        ) {
            let initializer = state_variable
                .value()
                .expect("constant state variable has no initializer");
            let target_type =
                TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
            return self.emit_value_for_target(&initializer, target_type, block);
        }
        let &(slot, byte_offset, location) = self
            .storage_layout
            .get(&state_variable.node_id())
            .expect("unregistered state variable");
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        let address = self.state.builder.emit_sol_addr_of(
            &crate::ast::contract::ContractEmitter::storage_symbol(slot, byte_offset, location),
            Self::address_type(&self.state.builder, element_type, location, &declared_type),
            &block,
        );
        let value = self
            .state
            .builder
            .emit_sol_load(address, element_type, &block)?;
        Ok((value, block))
    }

    /// Emits a conditional whose branches are tuples
    /// (`cond ? (a, b) : (c, d)`) as a vector of selected values — one `sol.if`
    /// over per-component result slots, mirroring the single-value conditional.
    ///
    /// Returns `Ok(None)` when the branches are not both tuples of equal,
    /// non-zero arity with resolvable element types, so callers fall through to
    /// their existing handling.
    pub fn emit_conditional_tuple_values(
        &self,
        conditional: &slang_solidity_v2::ast::ConditionalExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let (
            Expression::TupleExpression(true_tuple),
            Expression::TupleExpression(false_tuple),
        ) = (conditional.true_expression(), conditional.false_expression())
        else {
            return Ok(None);
        };
        let true_items: Vec<Expression> = true_tuple
            .items()
            .iter()
            .filter_map(|item| item.expression())
            .collect();
        let false_items: Vec<Expression> = false_tuple
            .items()
            .iter()
            .filter_map(|item| item.expression())
            .collect();
        if true_items.is_empty() || true_items.len() != false_items.len() {
            return Ok(None);
        }
        let mut result_types = Vec::with_capacity(true_items.len());
        for item in &true_items {
            let Some(item_type) = self.resolve_slang_type(item.get_type()) else {
                return Ok(None);
            };
            result_types.push(item_type);
        }

        let (condition_value, block) = self.emit_value(&conditional.operand(), block)?;
        let condition_boolean = self.emit_is_nonzero(condition_value, &block);
        let slots: Vec<Value<'context, 'block>> = result_types
            .iter()
            .map(|&result_type| self.state.builder.emit_sol_alloca(result_type, &block))
            .collect();
        let (then_block, else_block) = self.state.builder.emit_sol_if(condition_boolean, &block);

        for (branch_block, items) in [(then_block, &true_items), (else_block, &false_items)] {
            let mut current = branch_block;
            for (index, item) in items.iter().enumerate() {
                let (value, next) = self.emit_value(item, current)?;
                current = next;
                let cast = TypeConversion::from_target_type(result_types[index], &self.state.builder)
                    .emit(value, &self.state.builder, &current);
                self.state.builder.emit_sol_store(cast, slots[index], &current);
            }
            self.state.builder.emit_sol_yield(&current);
        }

        let mut values = Vec::with_capacity(slots.len());
        for (index, &slot) in slots.iter().enumerate() {
            values.push(self.state.builder.emit_sol_load(slot, result_types[index], &block)?);
        }
        Ok(Some((values, block)))
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

    /// If `expression` is a bare function name (always an internal function
    /// pointer), returns its `!sol.func_ref` type built from the declared
    /// signature. Slang types such a reference from the function's `Public`
    /// visibility, which `resolve_slang_type` maps to `ext_func_ref`; callers
    /// that *infer* a result type from the expression (array literals,
    /// ternaries) use this to recover the authoritative internal-pointer type.
    fn bare_function_ref_type(&self, expression: &Expression) -> Option<Type<'context>> {
        let Expression::Identifier(identifier) = expression else {
            return None;
        };
        let Some(Definition::Function(function_definition)) =
            identifier.resolve_to_definition()
        else {
            return None;
        };
        let (_, parameter_types, return_types) = self
            .state
            .resolve_function(function_definition.node_id())
            .ok()?;
        Some(self.state.builder.types.func_ref(parameter_types, return_types))
    }

    /// Whether an expression is a computed form worth constant-folding (binary
    /// arithmetic / bitwise / shift / unary). Simple literals, identifiers,
    /// and calls are excluded — they keep their existing emission paths.
    fn is_foldable_expression(expression: &Expression) -> bool {
        matches!(
            expression,
            Expression::AdditiveExpression(_)
                | Expression::MultiplicativeExpression(_)
                | Expression::ExponentiationExpression(_)
                | Expression::ShiftExpression(_)
                | Expression::BitwiseAndExpression(_)
                | Expression::BitwiseOrExpression(_)
                | Expression::BitwiseXorExpression(_)
                | Expression::PrefixExpression(_)
        )
    }

    /// Emits a folded constant value when `expression` carries a compile-time
    /// `Literal` integer/rational type. Returns `None` when the expression is
    /// not a constant (slang only assigns a `Literal` type to compile-time
    /// constants) or the rational is non-integer.
    fn try_emit_constant_literal(
        &self,
        expression: &Expression,
        block: &BlockRef<'context, 'block>,
    ) -> Option<Value<'context, 'block>> {
        use slang_solidity_v2::ast::LiteralKind;
        let SlangType::Literal(literal_type) = expression.get_type()? else {
            return None;
        };
        let value = match literal_type.kind() {
            LiteralKind::Integer { value } => value,
            LiteralKind::HexInteger { value, .. } => value,
            LiteralKind::Rational { value } => {
                if !value.is_integer() {
                    return None;
                }
                value.to_integer()
            }
            _ => return None,
        };
        let result_type = self.resolve_slang_type(expression.get_type())?;
        Some(self.state.builder.emit_constant(&value, result_type, block))
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
