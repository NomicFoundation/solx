//!
//! Solidity operator parsed from source text.
//!

use melior::ir::BlockRef;

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
    pub fn emit<'context, 'block>(
        self,
        checked: bool,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        match self {
            Self::Add | Self::Increment => lhs.add(rhs, checked, context, block),
            Self::Subtract | Self::Decrement => lhs.subtract(rhs, checked, context, block),
            Self::Multiply => lhs.multiply(rhs, checked, context, block),
            Self::Divide => lhs.divide(rhs, checked, context, block),
            Self::Remainder => lhs.remainder(rhs, context, block),
            Self::Exponentiation => lhs.exponentiate(rhs, checked, context, block),
            Self::BitwiseAnd => lhs.bitand(rhs, context, block),
            Self::BitwiseOr => lhs.bitor(rhs, context, block),
            Self::BitwiseXor => lhs.bitxor(rhs, context, block),
            Self::ShiftLeft => lhs.shl(rhs, context, block),
            Self::ShiftRight => lhs.shr(rhs, context, block),
            _ => unreachable!("emit called on non-arithmetic operator: {self:?}"),
        }
    }
}
