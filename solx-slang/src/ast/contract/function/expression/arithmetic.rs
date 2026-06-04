//!
//! Binary arithmetic expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::AdditiveExpressionOperator;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::MultiplicativeExpressionOperator;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PostfixExpressionOperator;
use slang_solidity_v2::ast::PrefixExpression;
use slang_solidity_v2::ast::PrefixExpressionOperator;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;
use solx_utils::DataLocation;

use solx_mlir::Builder;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// A binary arithmetic operation, abstracting over the source operator so the
/// checked/unchecked Sol op selection lives in one place.
#[derive(Debug, Clone, Copy)]
pub enum ArithmeticOperation {
    /// `+`
    Add,
    /// `-`
    Subtract,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Remainder,
    /// `**`
    Exponentiation,
}

impl ArithmeticOperation {
    /// Emits this operator's Sol op through the builder and returns its result.
    ///
    /// In checked mode (Solidity 0.8+ default) the overflow-trapping variants
    /// are emitted; inside `unchecked {}` the wrapping variants are. `%` has no
    /// checked variant.
    pub fn emit<'context, 'block>(
        self,
        checked: bool,
        builder: &Builder<'context>,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match (self, checked) {
            (Self::Add, true) => builder.emit_sol_cadd(lhs, rhs, block),
            (Self::Add, false) => builder.emit_sol_add(lhs, rhs, block),
            (Self::Subtract, true) => builder.emit_sol_csub(lhs, rhs, block),
            (Self::Subtract, false) => builder.emit_sol_sub(lhs, rhs, block),
            (Self::Multiply, true) => builder.emit_sol_cmul(lhs, rhs, block),
            (Self::Multiply, false) => builder.emit_sol_mul(lhs, rhs, block),
            (Self::Divide, true) => builder.emit_sol_cdiv(lhs, rhs, block),
            (Self::Divide, false) => builder.emit_sol_div(lhs, rhs, block),
            (Self::Remainder, _) => builder.emit_sol_mod(lhs, rhs, block),
            (Self::Exponentiation, true) => builder.emit_sol_cexp(lhs, rhs, block),
            (Self::Exponentiation, false) => builder.emit_sol_exp(lhs, rhs, block),
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an additive expression (`+`, `-`).
    pub fn emit_additive(
        &self,
        expression: &AdditiveExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            AdditiveExpressionOperator::Plus(_) => ArithmeticOperation::Add,
            AdditiveExpressionOperator::Minus(_) => ArithmeticOperation::Subtract,
        };
        let result_type = expression
            .get_type()
            .expect("binder types every arithmetic expression");
        self.emit_binary_arithmetic(
            operation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers a multiplicative expression (`*`, `/`, `%`).
    pub fn emit_multiplicative(
        &self,
        expression: &MultiplicativeExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            MultiplicativeExpressionOperator::Asterisk(_) => ArithmeticOperation::Multiply,
            MultiplicativeExpressionOperator::Slash(_) => ArithmeticOperation::Divide,
            MultiplicativeExpressionOperator::Percent(_) => ArithmeticOperation::Remainder,
        };
        let result_type = expression
            .get_type()
            .expect("binder types every arithmetic expression");
        self.emit_binary_arithmetic(
            operation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Lowers an exponentiation expression (`**`).
    pub fn emit_exponentiation(
        &self,
        expression: &ExponentiationExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = expression
            .get_type()
            .expect("binder types every arithmetic expression");
        self.emit_binary_arithmetic(
            ArithmeticOperation::Exponentiation,
            &expression.left_operand(),
            &expression.right_operand(),
            &result_type,
            block,
        )
    }

    /// Emits a binary arithmetic operation.
    ///
    /// Both operands are coerced to the expression's binder-assigned type so
    /// the Sol op satisfies `SameOperandsAndResultType` and matches solc's
    /// type-annotated IR.
    fn emit_binary_arithmetic(
        &self,
        operation: ArithmeticOperation,
        left: &Expression,
        right: &Expression,
        result_slang_type: &SlangType,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type =
            TypeConversion::resolve_slang_type(result_slang_type, None, &self.state.builder);
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, result_type, block)?;
        let value = operation.emit(self.checked, &self.state.builder, lhs, rhs, &block);
        Ok((value, block))
    }

    /// Emits both operands of a binary expression — right-to-left, matching
    /// solc's evaluation order — and coerces each to `result_type` so the Sol
    /// op satisfies `SameOperandsAndResultType`. Shared with the bitwise domain.
    pub fn emit_binary_operands(
        &self,
        left: &Expression,
        right: &Expression,
        result_type: Type<'context>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (rhs, block) = self.emit_value(right, block)?;
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        Ok((lhs, rhs, block))
    }

    /// Lowers a postfix step (`x++`, `x--`), yielding the value before the step.
    pub fn emit_postfix(
        &self,
        expression: &PostfixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let operation = match expression.operator() {
            PostfixExpressionOperator::PlusPlus(_) => ArithmeticOperation::Add,
            PostfixExpressionOperator::MinusMinus(_) => ArithmeticOperation::Subtract,
        };
        let (old, _new, block) =
            self.emit_increment_decrement(operation, &expression.operand(), block)?;
        Ok((old, block))
    }

    /// Lowers a prefix operator, routing each to its domain: `++`/`--` step
    /// here, `!` to logical, `~` to bitwise, `-` to negation. `delete` defers.
    pub fn emit_prefix(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let step = match expression.operator() {
            PrefixExpressionOperator::PlusPlus(_) => ArithmeticOperation::Add,
            PrefixExpressionOperator::MinusMinus(_) => ArithmeticOperation::Subtract,
            PrefixExpressionOperator::Bang(_) => {
                return self.emit_not(&expression.operand(), block);
            }
            PrefixExpressionOperator::Tilde(_) => return self.emit_bitwise_not(expression, block),
            PrefixExpressionOperator::Minus(_) => return self.emit_negate(expression, block),
            PrefixExpressionOperator::DeleteKeyword(_) => {
                return self.emit_delete(&expression.operand(), block);
            }
        };
        let (_old, new, block) =
            self.emit_increment_decrement(step, &expression.operand(), block)?;
        Ok((new, block))
    }

    /// Emits `delete <operand>`, resetting the target to its type's zero value.
    /// `delete m[k]` / `delete arr[i]` and `delete s.field` reset the addressed
    /// element in place; a bare identifier dispatches to the local- or
    /// state-variable handler.
    fn emit_delete(
        &self,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // `delete m[k]` / `delete arr[i]` resets the indexed element to zero.
        if let Expression::IndexAccessExpression(index_access) = operand {
            let (address, element_type, block) =
                self.emit_index_access_address(index_access, block)?;
            let zero = self
                .state
                .builder
                .emit_sol_constant(0, element_type, &block);
            self.state.builder.emit_sol_store(zero, address, &block);
            return Ok((zero, block));
        }
        // `delete s.field` resets the addressed struct field: a reference-typed
        // field (nested array / struct / `bytes` in storage) recurses through
        // `sol.delete`; a value-typed field stores its zero.
        if let Expression::MemberAccessExpression(access) = operand
            && let Some((address, element_type, block)) =
                self.emit_struct_field_address(access, block)?
        {
            let builder = &self.state.builder;
            if solx_mlir::TypeFactory::is_sol_reference(element_type) {
                builder.emit_sol_delete(address, &block);
                let placeholder = builder.emit_sol_constant(0, builder.types.ui256, &block);
                return Ok((placeholder, block));
            }
            let zero = self.delete_zero_value(element_type, &block);
            builder.emit_sol_store(zero, address, &block);
            return Ok((zero, block));
        }
        // `delete x` resets `x` to its type's zero value; reference-type deletion
        // needs storage-class-specific lowering, so dispatch on the definition.
        let Expression::Identifier(identifier) = operand else {
            unimplemented!("delete of a non-identifier operand");
        };
        let name = identifier.name();
        match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                self.emit_delete_local_variable(&name, identifier.get_type(), block)
            }
            Some(Definition::StateVariable(state_variable)) => {
                self.emit_delete_state_variable(&state_variable, block)
            }
            _ => unimplemented!("unsupported delete target: {name}"),
        }
    }

    /// The zero value for a value-typed slot: a zero integer, an enum's zero
    /// variant (bridged through `sol.enum_cast`), or a `ui256` zero coerced to
    /// the type (`address(0)`, `bytesN(0)`, `false`).
    fn delete_zero_value(
        &self,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        if IntegerType::try_from(element_type).is_ok() {
            builder.emit_sol_constant(0, element_type, block)
        } else if solx_mlir::TypeFactory::is_sol_enum(element_type) {
            let raw = builder.emit_sol_constant(0, builder.types.ui256, block);
            builder.emit_sol_enum_cast(raw, element_type, block)
        } else {
            let raw = builder.emit_sol_constant(0, builder.types.ui256, block);
            TypeConversion::from_target_type(element_type, builder).emit(raw, builder, block)
        }
    }

    /// Emits `delete x` for a local variable / parameter, rebinding it to its
    /// type's zero value: integers to `0`, enums to their zero variant,
    /// function pointers to the default pointer, dynamic aggregates (arrays /
    /// `bytes` / `string`) to a fresh empty allocation, and fixed aggregates
    /// (structs / fixed arrays) to a zero-initialised allocation.
    fn emit_delete_local_variable(
        &self,
        name: &str,
        slang_type: Option<SlangType>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (pointer, element_type) = self.environment.variable_with_type(name);
        let builder = &self.state.builder;
        let zero = if IntegerType::try_from(element_type).is_ok() {
            builder.emit_sol_constant(0, element_type, &block)
        } else if solx_mlir::TypeFactory::is_sol_enum(element_type) {
            let raw = builder.emit_sol_constant(0, builder.types.ui256, &block);
            builder.emit_sol_enum_cast(raw, element_type, &block)
        } else {
            match slang_type {
                Some(SlangType::Function(_)) => {
                    builder.emit_sol_default_func_constant(element_type, &block)
                }
                Some(SlangType::Array(_) | SlangType::String(_) | SlangType::Bytes(_)) => {
                    let zero_size = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    builder.emit_sol_malloc_sized(element_type, zero_size, &block)
                }
                Some(SlangType::FixedSizeArray(_) | SlangType::Struct(_)) => {
                    builder.emit_sol_malloc(element_type, &block)
                }
                _ => unimplemented!("delete on a non-integer local '{name}' is not yet supported"),
            }
        };
        builder.emit_sol_store(zero, pointer, &block);
        Ok((zero, block))
    }

    /// Emits `delete x` for a state variable, resetting its storage to the
    /// default. A mapping is a no-op; dynamic `bytes` / `string` reset by
    /// copying a fresh empty memory buffer into the slot; arrays / structs
    /// recurse through `sol.delete`; value types and enums store a zero word.
    fn emit_delete_state_variable(
        &self,
        state_variable: &StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let declared_type = state_variable
            .get_type()
            .expect("binder types every state variable");
        let element_type =
            TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .expect("every state variable has a storage slot")
            .clone();
        let builder = &self.state.builder;

        if declared_type.is_reference_type() {
            match &declared_type {
                // `delete` on a mapping is a no-op in Solidity.
                SlangType::Mapping(_) => {}
                // Dynamic `bytes` / `string` reset to empty: copy a freshly
                // allocated zero-length memory buffer into the slot (`sol.copy`
                // writes the destination and clears the previous tail).
                SlangType::Bytes(_) | SlangType::String(_) => {
                    let memory_type = TypeConversion::resolve_slang_type(
                        &declared_type,
                        Some(DataLocation::Memory),
                        builder,
                    );
                    let zero_size = builder.emit_sol_constant(0, builder.types.ui256, &block);
                    let default_value =
                        builder.emit_sol_malloc_sized(memory_type, zero_size, &block);
                    let address = builder.emit_sol_addr_of(&slot.name, element_type, &block);
                    builder.emit_sol_copy(default_value, address, &block);
                }
                // Arrays and structs: `sol.delete` recursively clears every
                // storage slot the aggregate occupies.
                SlangType::Struct(_) | SlangType::Array(_) | SlangType::FixedSizeArray(_) => {
                    let address = builder.emit_sol_addr_of(&slot.name, element_type, &block);
                    builder.emit_sol_delete(address, &block);
                }
                _ => unimplemented!(
                    "delete of this reference-type storage variable is not yet supported"
                ),
            }
            // The value of a `delete` expression is rarely consumed.
            let result = builder.emit_sol_constant(0, builder.types.ui256, &block);
            return Ok((result, block));
        }

        let zero = self.delete_zero_value(element_type, &block);
        self.emit_storage_store(&slot, zero, element_type, &block);
        Ok((zero, block))
    }

    /// Lowers unary negation `-x` as `0 - x` (unchecked subtraction; checked
    /// negation would need signed-type-aware handling of `-INT_MIN`).
    fn emit_negate(
        &self,
        expression: &PrefixExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = TypeConversion::resolve_slang_type(
            &expression
                .get_type()
                .expect("binder types every negation expression"),
            None,
            &self.state.builder,
        );
        let (value, block) = self.emit_value(&expression.operand(), block)?;
        let value = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        let zero = self.state.builder.emit_sol_constant(0, result_type, &block);
        let result = self.state.builder.emit_sol_sub(zero, value, &block);
        Ok((result, block))
    }

    /// Emits a `±1` read-modify-write of an lvalue, returning both the value
    /// before the step and the value after it.
    fn emit_increment_decrement(
        &self,
        operation: ArithmeticOperation,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (lvalue, block) = self.resolve_lvalue(operand, block)?;
        let element_type = lvalue.element_type();
        let old = self.emit_lvalue_load(&lvalue, &block)?;
        let one = self
            .state
            .builder
            .emit_sol_constant(1, element_type, &block);
        let new = operation.emit(self.checked, &self.state.builder, old, one, &block);
        self.emit_lvalue_store(&lvalue, new, &block);
        Ok((old, new, block))
    }
}
