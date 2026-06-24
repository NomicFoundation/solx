//!
//! Solidity operator, bridged from slang's typed per-expression operator enums.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::AddOperation;
use solx_mlir::ods::sol::AndOperation;
use solx_mlir::ods::sol::CAddOperation;
use solx_mlir::ods::sol::CDivOperation;
use solx_mlir::ods::sol::CExpOperation;
use solx_mlir::ods::sol::CMulOperation;
use solx_mlir::ods::sol::CSubOperation;
use solx_mlir::ods::sol::DivOperation;
use solx_mlir::ods::sol::ExpOperation;
use solx_mlir::ods::sol::ModOperation;
use solx_mlir::ods::sol::MulOperation;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::OrOperation;
use solx_mlir::ods::sol::ShlOperation;
use solx_mlir::ods::sol::ShrOperation;
use solx_mlir::ods::sol::SubOperation;
use solx_mlir::ods::sol::XorOperation;

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitPlace;
use crate::ast::Place;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;

/// Solidity operator, bridged from slang's typed per-expression operator enums (never parsed from text).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// `+` (binary)
    Add,
    /// `-` (binary or unary negation)
    Subtract,
    /// `*`
    Multiply,
    /// `/`
    Divide,
    /// `%`
    Remainder,
    /// `**`
    Exponentiation,

    /// `&`
    BitwiseAnd,
    /// `|`
    BitwiseOr,
    /// `^`
    BitwiseXor,
    /// `<<`
    ShiftLeft,
    /// `>>` (and the no-op `>>>`)
    ShiftRight,
    /// `~`
    BitwiseNot,

    /// `!`
    Not,

    /// `++`
    Increment,
    /// `--`
    Decrement,
}

impl Operator {
    /// Builds a Sol dialect binary operation. Checked mode uses the `sol.c*` arithmetic variants;
    /// modulo, bitwise, and shift are always unchecked.
    pub fn build_binary_operation<'context>(
        self,
        mode: ArithmeticMode,
        builder: &Builder<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        let checked = matches!(mode, ArithmeticMode::Checked);
        match self {
            Self::Add | Self::Increment if checked => {
                mlir_op_build!(builder, CAddOperation.lhs(lhs).rhs(rhs))
            }
            Self::Add | Self::Increment => mlir_op_build!(builder, AddOperation.lhs(lhs).rhs(rhs)),
            Self::Subtract | Self::Decrement if checked => {
                mlir_op_build!(builder, CSubOperation.lhs(lhs).rhs(rhs))
            }
            Self::Subtract | Self::Decrement => {
                mlir_op_build!(builder, SubOperation.lhs(lhs).rhs(rhs))
            }
            Self::Multiply if checked => mlir_op_build!(builder, CMulOperation.lhs(lhs).rhs(rhs)),
            Self::Multiply => mlir_op_build!(builder, MulOperation.lhs(lhs).rhs(rhs)),
            Self::Divide if checked => mlir_op_build!(builder, CDivOperation.lhs(lhs).rhs(rhs)),
            Self::Divide => mlir_op_build!(builder, DivOperation.lhs(lhs).rhs(rhs)),
            Self::Remainder => mlir_op_build!(builder, ModOperation.lhs(lhs).rhs(rhs)),
            Self::Exponentiation if checked => {
                mlir_op_build!(
                    builder,
                    CExpOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs)
                )
            }
            Self::Exponentiation => {
                mlir_op_build!(builder, ExpOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            Self::BitwiseAnd => mlir_op_build!(builder, AndOperation.lhs(lhs).rhs(rhs)),
            Self::BitwiseOr => mlir_op_build!(builder, OrOperation.lhs(lhs).rhs(rhs)),
            Self::BitwiseXor => mlir_op_build!(builder, XorOperation.lhs(lhs).rhs(rhs)),
            // `sol.shl`/`sol.shr` have a free `rhs` (independent shift amount), so the result
            // type is not inferable from both operands and is set explicitly to follow `lhs`.
            Self::ShiftLeft => {
                mlir_op_build!(builder, ShlOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            Self::ShiftRight => {
                mlir_op_build!(builder, ShrOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            _ => unreachable!(
                "build_binary_operation called on non-arithmetic operator: {self:?}"
            ),
        }
    }

    /// Lowers a binary expression `left <op> right` to its result value (dispatching to a bound
    /// user-defined operator when present). With no `target_type`, the wider operand type is selected.
    pub fn emit_binary<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        left: &Expression,
        right: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> (AstValue<'context, 'block>, BlockRef<'context, 'block>) {
        let BlockAnd { value: rhs, block } = right.emit(context, block);
        let BlockAnd { value: lhs, block } = left.emit(context, block);
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = lhs.r#type().integer_bit_width();
            let rhs_width = rhs.r#type().integer_bit_width();
            if lhs_width >= rhs_width {
                lhs.r#type().into_mlir()
            } else {
                rhs.r#type().into_mlir()
            }
        });
        let value = self.emit_value_binary(
            context.arithmetic_mode,
            &context.state.builder,
            lhs,
            rhs,
            result_type,
            &block,
        );
        (value, block)
    }

    /// Combines already-materialised `lhs`/`rhs` into a value of `result_type`, casting both
    /// operands to `result_type` before the operation.
    pub fn emit_value_binary<'context, 'block>(
        self,
        mode: ArithmeticMode,
        builder: &Builder<'context>,
        lhs: AstValue<'context, 'block>,
        rhs: AstValue<'context, 'block>,
        result_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> AstValue<'context, 'block> {
        let lhs = lhs
            .cast(AstType::new(result_type), builder, block)
            .into_mlir();
        let rhs = rhs
            .cast(AstType::new(result_type), builder, block)
            .into_mlir();
        let result: Value<'context, 'block> = block
            .append_operation(self.build_binary_operation(mode, builder, lhs, rhs))
            .result(0)
            .expect("binary operation always produces one result")
            .into();
        result.into()
    }

    /// Emits postfix `++` / `--`, returning the old value.
    pub fn emit_postfix<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> (AstValue<'context, 'block>, BlockRef<'context, 'block>) {
        if let Some((old, _new, block)) =
            self.emit_increment_decrement_indexed(context, operand, block)
        {
            return (old.into(), block);
        }
        let (old, _) = self.emit_increment_decrement(context, operand, &block);
        (old.into(), block)
    }

    /// Emits prefix operators (`!`, `-`, `~`, `++`, `--`), dispatching a `-` / `~` on a bound
    /// user-defined value type to its function. `target_type`, when set, types the operation.
    pub fn emit_prefix<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> (AstValue<'context, 'block>, BlockRef<'context, 'block>) {
        match self {
            Operator::Increment | Operator::Decrement => {
                if let Some((_old, new_value, block)) =
                    self.emit_increment_decrement_indexed(context, operand, block)
                {
                    return (new_value.into(), block);
                }
                let (_old, new_value) = self.emit_increment_decrement(context, operand, &block);
                (new_value.into(), block)
            }
            Operator::BitwiseNot => {
                let BlockAnd { value, block } = operand.emit(context, block);
                let operand_type = target_type.expect("slang validated");
                let value = value
                    .cast(AstType::new(operand_type), &context.state.builder, &block)
                    .into_mlir();
                let result: Value<'context, 'block> =
                    mlir_op!(&context.state.builder, block, NotOperation.value(value));
                (result.into(), block)
            }
            Operator::Not => {
                let BlockAnd { value, block } = operand.emit(context, block);
                let zero = AstValue::constant(0, value.r#type(), &context.state.builder, &block);
                let cmp = value.compare(zero, CmpPredicate::Eq, &context.state.builder, &block);
                let result_type = target_type.unwrap_or(
                    AstType::unsigned(context.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir(),
                );
                let result = cmp.cast(AstType::new(result_type), &context.state.builder, &block);
                (result, block)
            }
            Operator::Subtract => {
                // Unary negation uses unchecked subtraction: checked negation (e.g. -INT_MIN reverting)
                // needs signed-type awareness and a dedicated op, not sol.csub.
                let BlockAnd { value, block } = operand.emit(context, block);
                let operand_type = target_type.expect("slang validated");
                let value = value
                    .cast(AstType::new(operand_type), &context.state.builder, &block)
                    .into_mlir();
                let zero = AstValue::constant(
                    0,
                    AstType::new(operand_type),
                    &context.state.builder,
                    &block,
                )
                .into_mlir();
                let result: Value<'context, 'block> = mlir_op!(
                    &context.state.builder,
                    block,
                    SubOperation.lhs(zero).rhs(value)
                );
                (result.into(), block)
            }
            _ => unimplemented!("unsupported prefix operator: {self:?}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)` for an
    /// identifier lvalue (a local / parameter or a state variable).
    fn emit_increment_decrement<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        block: &BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, Value<'context, 'block>) {
        let Expression::Identifier(identifier) = operand else {
            unimplemented!("unsupported operand for {self:?}");
        };

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = context
                    .storage_layout
                    .get(&state_variable.node_id())
                    .unwrap_or_else(|| {
                        unimplemented!("unregistered state variable {:?}", state_variable.node_id())
                    });
                let element_type = AstType::resolve_state_variable(
                    &state_variable.get_type().expect("slang validated"),
                    &context.state.builder,
                );
                let old = slot.load(&context.state.builder, element_type, block);
                let new_value = self.emit_step(context, old, element_type, block);
                slot.store(&context.state.builder, new_value, element_type, block);
                (old, new_value)
            }
            Some(definition @ (Definition::Variable(_) | Definition::Parameter(_))) => {
                let pointer = Pointer::new(context.environment.variable(definition.node_id()));
                let element_type = pointer.pointee();
                let old = pointer
                    .load(element_type, &context.state.builder, block)
                    .into_mlir();
                let new_value = self.emit_step(context, old, element_type.into_mlir(), block);
                pointer.store(AstValue::new(new_value), &context.state.builder, block);
                (old, new_value)
            }
            None => unreachable!("slang resolves every identifier reference"),
            Some(other) => {
                unimplemented!("unsupported operand for {self:?}: {:?}", other.node_id())
            }
        }
    }

    /// Emits `++` / `--` on a *computed* lvalue (`a[i]` or a struct field); `None` for any other operand.
    fn emit_increment_decrement_indexed<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> Option<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let (address, element_type, block) = match operand {
            Expression::IndexAccessExpression(index_access) => {
                let BlockAnd {
                    value:
                        Place {
                            address,
                            element_type,
                        },
                    block,
                } = index_access.emit_place(context, block);
                (address, element_type, block)
            }
            Expression::MemberAccessExpression(access) => {
                let BlockAnd {
                    value:
                        Place {
                            address,
                            element_type,
                        },
                    block,
                } = access.emit_place(context, block);
                (address, element_type, block)
            }
            _ => return None,
        };
        let pointer = Pointer::new(address);
        let old = pointer
            .load(AstType::new(element_type), &context.state.builder, &block)
            .into_mlir();
        let new_value = self.emit_step(context, old, element_type, &block);
        pointer.store(AstValue::new(new_value), &context.state.builder, &block);
        Some((old, new_value, block))
    }

    /// Applies the `++` / `--` step to a loaded value: adds or subtracts a typed `1`, honoring the arithmetic mode.
    fn emit_step<'context, 'block>(
        self,
        context: &ExpressionContext<'_, 'context, 'block>,
        old: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let one = AstValue::constant(1, AstType::new(element_type), &context.state.builder, block)
            .into_mlir();
        block
            .append_operation(self.build_binary_operation(
                context.arithmetic_mode,
                &context.state.builder,
                old,
                one,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into()
    }
}
