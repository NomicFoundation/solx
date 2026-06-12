//!
//! Solidity operator, bridged from slang's typed per-expression operator enums.
//!

use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;

use solx_mlir::Builder;
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
use solx_mlir::ods::sol::OrOperation;
use solx_mlir::ods::sol::ShlOperation;
use solx_mlir::ods::sol::ShrOperation;
use solx_mlir::ods::sol::SubOperation;
use solx_mlir::ods::sol::XorOperation;

use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;

/// Solidity operator, bridged from slang's typed per-expression operator enums
/// (`AdditiveExpressionOperator`, `ShiftExpressionOperator`, …) — never parsed
/// from source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    // ---- Arithmetic ----
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

    // ---- Bitwise ----
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

    // ---- Logical ----
    /// `!`
    Not,

    // ---- Step ----
    /// `++`
    Increment,
    /// `--`
    Decrement,
}

impl Operator {
    /// Builds a Sol dialect binary operation via ODS-generated builders.
    ///
    /// In [`ArithmeticMode::Checked`] mode, uses checked variants (`sol.cadd`,
    /// `sol.csub`, `sol.cmul`, `sol.cdiv`, `sol.cexp`) for arithmetic operators.
    /// Modulo, bitwise, and shift operators are always unchecked. Result type is
    /// inferred from `lhs` (`SameOperandsAndResultType`).
    ///
    /// # Panics
    ///
    /// Panics if called on a unary-only operator (`Not` / `BitwiseNot`), which
    /// the prefix emitter handles instead.
    pub fn emit_sol_binary_operation<'context>(
        self,
        mode: ArithmeticMode,
        builder: &Builder<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        let checked = matches!(mode, ArithmeticMode::Checked);
        match self {
            Self::Add | Self::Increment if checked => {
                sol_op_build!(builder, CAddOperation.lhs(lhs).rhs(rhs))
            }
            Self::Add | Self::Increment => sol_op_build!(builder, AddOperation.lhs(lhs).rhs(rhs)),
            Self::Subtract | Self::Decrement if checked => {
                sol_op_build!(builder, CSubOperation.lhs(lhs).rhs(rhs))
            }
            Self::Subtract | Self::Decrement => {
                sol_op_build!(builder, SubOperation.lhs(lhs).rhs(rhs))
            }
            Self::Multiply if checked => sol_op_build!(builder, CMulOperation.lhs(lhs).rhs(rhs)),
            Self::Multiply => sol_op_build!(builder, MulOperation.lhs(lhs).rhs(rhs)),
            Self::Divide if checked => sol_op_build!(builder, CDivOperation.lhs(lhs).rhs(rhs)),
            Self::Divide => sol_op_build!(builder, DivOperation.lhs(lhs).rhs(rhs)),
            Self::Remainder => sol_op_build!(builder, ModOperation.lhs(lhs).rhs(rhs)),
            Self::Exponentiation if checked => {
                sol_op_build!(
                    builder,
                    CExpOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs)
                )
            }
            Self::Exponentiation => {
                sol_op_build!(builder, ExpOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            Self::BitwiseAnd => sol_op_build!(builder, AndOperation.lhs(lhs).rhs(rhs)),
            Self::BitwiseOr => sol_op_build!(builder, OrOperation.lhs(lhs).rhs(rhs)),
            Self::BitwiseXor => sol_op_build!(builder, XorOperation.lhs(lhs).rhs(rhs)),
            // `sol.shl`/`sol.shr` now accept a `bytesN` (or integer) value with an
            // independent integer shift amount (`AllTypesMatch<lhs, result>`, rhs
            // free), so the result type is no longer inferable from both operands
            // and must be set explicitly — it follows the shifted value (`lhs`).
            Self::ShiftLeft => {
                sol_op_build!(builder, ShlOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            Self::ShiftRight => {
                sol_op_build!(builder, ShrOperation.result(lhs.r#type()).lhs(lhs).rhs(rhs))
            }
            _ => unreachable!(
                "emit_sol_binary_operation called on non-arithmetic operator: {self:?}"
            ),
        }
    }
}
