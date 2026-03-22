//!
//! Solidity operator parsed from source text.
//!

use std::str::FromStr;

use solx_mlir::ICmpPredicate;

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

    // ---- Step ----
    /// `++`
    Increment,
    /// `--`
    Decrement,
}

impl Operator {
    /// Returns the underlying arithmetic operator for compound assignment variants.
    ///
    /// `AddAssign` → `Add`, `SubtractAssign` → `Subtract`, etc.
    /// Non-assignment operators are returned unchanged.
    pub fn arithmetic_operator(self) -> Self {
        match self {
            Self::AddAssign => Self::Add,
            Self::SubtractAssign => Self::Subtract,
            Self::MultiplyAssign => Self::Multiply,
            Self::DivideAssign => Self::Divide,
            Self::RemainderAssign => Self::Remainder,
            Self::BitwiseAndAssign => Self::BitwiseAnd,
            Self::BitwiseOrAssign => Self::BitwiseOr,
            Self::BitwiseXorAssign => Self::BitwiseXor,
            Self::ShiftLeftAssign => Self::ShiftLeft,
            Self::ShiftRightAssign => Self::ShiftRight,
            other => other,
        }
    }

    /// Returns the LLVM dialect operation name for arithmetic, bitwise, and step operators.
    ///
    /// Selects signed variants (`sdiv`, `srem`, `ashr`) when `signed` is true.
    ///
    /// # Panics
    ///
    /// Panics if called on a comparison or assignment operator.
    pub fn llvm_operation_name(self, signed: bool) -> &'static str {
        match (self, signed) {
            (Self::Add | Self::Increment, _) => solx_mlir::Builder::ADD,
            (Self::Subtract | Self::Decrement, _) => solx_mlir::Builder::SUB,
            (Self::Multiply, _) => solx_mlir::Builder::MUL,
            (Self::Divide, true) => solx_mlir::Builder::SDIV,
            (Self::Divide, false) => solx_mlir::Builder::UDIV,
            (Self::Remainder, true) => solx_mlir::Builder::SREM,
            (Self::Remainder, false) => solx_mlir::Builder::UREM,
            (Self::BitwiseAnd, _) => solx_mlir::Builder::AND,
            (Self::BitwiseOr, _) => solx_mlir::Builder::OR,
            (Self::BitwiseXor, _) => solx_mlir::Builder::XOR,
            (Self::ShiftLeft, _) => solx_mlir::Builder::SHL,
            (Self::ShiftRight, true) => solx_mlir::Builder::ASHR,
            (Self::ShiftRight, false) => solx_mlir::Builder::LSHR,
            _ => unreachable!("llvm_operation_name called on non-arithmetic operator: {self:?}"),
        }
    }

    /// Returns the ICmp predicate for comparison operators.
    ///
    /// Selects signed variants (`sgt`, `sge`, `slt`, `sle`) when `signed` is true.
    ///
    /// # Panics
    ///
    /// Panics if called on a non-comparison operator.
    pub fn icmp_predicate(self, signed: bool) -> ICmpPredicate {
        match (self, signed) {
            (Self::Equal, _) => ICmpPredicate::Eq,
            (Self::NotEqual, _) => ICmpPredicate::Ne,
            (Self::Greater, false) => ICmpPredicate::Ugt,
            (Self::Greater, true) => ICmpPredicate::Sgt,
            (Self::GreaterEqual, false) => ICmpPredicate::Uge,
            (Self::GreaterEqual, true) => ICmpPredicate::Sge,
            (Self::Less, false) => ICmpPredicate::Ult,
            (Self::Less, true) => ICmpPredicate::Slt,
            (Self::LessEqual, false) => ICmpPredicate::Ule,
            (Self::LessEqual, true) => ICmpPredicate::Sle,
            _ => unreachable!("icmp_predicate called on non-comparison operator: {self:?}"),
        }
    }
}

impl FromStr for Operator {
    type Err = anyhow::Error;

    fn from_str(operator: &str) -> Result<Self, Self::Err> {
        match operator {
            "+" => Ok(Self::Add),
            "-" => Ok(Self::Subtract),
            "*" => Ok(Self::Multiply),
            "/" => Ok(Self::Divide),
            "%" => Ok(Self::Remainder),
            "+=" => Ok(Self::AddAssign),
            "-=" => Ok(Self::SubtractAssign),
            "*=" => Ok(Self::MultiplyAssign),
            "/=" => Ok(Self::DivideAssign),
            "%=" => Ok(Self::RemainderAssign),
            "&" => Ok(Self::BitwiseAnd),
            "|" => Ok(Self::BitwiseOr),
            "^" => Ok(Self::BitwiseXor),
            "<<" => Ok(Self::ShiftLeft),
            ">>" => Ok(Self::ShiftRight),
            "&=" => Ok(Self::BitwiseAndAssign),
            "|=" => Ok(Self::BitwiseOrAssign),
            "^=" => Ok(Self::BitwiseXorAssign),
            "<<=" => Ok(Self::ShiftLeftAssign),
            ">>=" => Ok(Self::ShiftRightAssign),
            "==" => Ok(Self::Equal),
            "!=" => Ok(Self::NotEqual),
            ">" => Ok(Self::Greater),
            ">=" => Ok(Self::GreaterEqual),
            "<" => Ok(Self::Less),
            "<=" => Ok(Self::LessEqual),
            "++" => Ok(Self::Increment),
            "--" => Ok(Self::Decrement),
            _ => anyhow::bail!("unsupported operator: {operator}"),
        }
    }
}
