//!
//! Solidity operator parsed from source text.
//!

use solx_mlir::Context;
use solx_mlir::Value;

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
    /// Emits this binary operation over `lhs`/`rhs`, returning the result value.
    ///
    /// Dispatches to the matching [`Value`] constructor. When `checked` is true, the arithmetic
    /// operators use their reverting variants (`sol.cadd`, `sol.csub`, `sol.cmul`, `sol.cdiv`,
    /// `sol.cexp`); modulo, bitwise, and shift operators are always unchecked.
    ///
    /// # Panics
    ///
    /// Panics if called on a comparison or assignment operator.
    pub fn emit<'context>(
        self,
        checked: bool,
        lhs: Value<'context>,
        rhs: Value<'context>,
        context: &Context<'context>,
    ) -> Value<'context> {
        match self {
            Self::Add | Self::Increment => lhs.add(rhs, checked, context),
            Self::Subtract | Self::Decrement => lhs.subtract(rhs, checked, context),
            Self::Multiply => lhs.multiply(rhs, checked, context),
            Self::Divide => lhs.divide(rhs, checked, context),
            Self::Remainder => lhs.remainder(rhs, context),
            Self::Exponentiation => lhs.exponentiate(rhs, checked, context),
            Self::BitwiseAnd => lhs.bitand(rhs, context),
            Self::BitwiseOr => lhs.bitor(rhs, context),
            Self::BitwiseXor => lhs.bitxor(rhs, context),
            Self::ShiftLeft => lhs.shl(rhs, context),
            Self::ShiftRight => lhs.shr(rhs, context),
            _ => unreachable!("emit called on non-arithmetic operator: {self:?}"),
        }
    }
}
