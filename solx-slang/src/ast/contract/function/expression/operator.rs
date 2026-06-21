//!
//! Solidity operator parsed from source text.
//!

use melior::ir::Location;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::operation::Operation;

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

/// Solidity operator parsed from source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    // ---- Arithmetic ----
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

    // ---- Arithmetic assignment ----
    /// `+=`
    AddAssign,
    /// `-=`
    SubtractAssign,
    /// `*=`
    MultiplyAssign,
    /// `/=`
    DivideAssign,
    /// `%=`
    RemainderAssign,

    // ---- Bitwise ----
    /// `&`
    BitwiseAnd,
    /// `|`
    BitwiseOr,
    /// `^`
    BitwiseXor,
    /// `<<`
    ShiftLeft,
    /// `>>`
    ShiftRight,
    /// `~`
    BitwiseNot,

    // ---- Bitwise assignment ----
    /// `&=`
    BitwiseAndAssign,
    /// `|=`
    BitwiseOrAssign,
    /// `^=`
    BitwiseXorAssign,
    /// `<<=`
    ShiftLeftAssign,
    /// `>>=`
    ShiftRightAssign,

    // ---- Comparison ----
    /// `==`
    Equal,
    /// `!=`
    NotEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `!`
    Not,

    // ---- Step ----
    /// `++`
    Increment,
    /// `--`
    Decrement,

    // ---- Other ----
    /// `delete`
    Delete,
}

impl Operator {
    /// Builds a Sol dialect binary operation via ODS-generated builders.
    ///
    /// When `checked` is true, uses checked variants (`sol.cadd`, `sol.csub`,
    /// `sol.cmul`, `sol.cdiv`, `sol.cexp`) for arithmetic operators. Modulo, bitwise,
    /// and shift operators are always unchecked. Result type is inferred
    /// from `lhs` (`SameOperandsAndResultType`).
    ///
    /// # Panics
    ///
    /// Panics if called on a comparison or assignment operator.
    pub fn emit_sol_binary_operation<'context>(
        self,
        checked: bool,
        context: &'context melior::Context,
        location: Location<'context>,
        lhs: Value<'context, '_>,
        rhs: Value<'context, '_>,
    ) -> Operation<'context> {
        match self {
            Self::Add | Self::Increment if checked => CAddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Add | Self::Increment => AddOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Subtract | Self::Decrement if checked => {
                CSubOperation::builder(context, location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into()
            }
            Self::Subtract | Self::Decrement => SubOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Multiply if checked => CMulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Multiply => MulOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Divide if checked => CDivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Divide => DivOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Remainder => ModOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Exponentiation if checked => CExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::Exponentiation => ExpOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::BitwiseAnd => AndOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::BitwiseOr => OrOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::BitwiseXor => XorOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftLeft => ShlOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftRight => ShrOperation::builder(context, location)
                .result(lhs.r#type())
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            _ => unreachable!(
                "emit_sol_binary_operation called on non-arithmetic operator: {self:?}"
            ),
        }
    }
}
