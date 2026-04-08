//!
//! Solidity operator parsed from source text.
//!

use std::str::FromStr;

use solx_mlir::CmpPredicate;

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

    /// Returns the Sol dialect operation name for arithmetic, bitwise, and step operators.
    ///
    /// When `checked` is true, uses `sol.cadd`/`sol.csub`/`sol.cmul`/`sol.cdiv`
    /// for add/subtract/multiply/divide (Solidity 0.8+ default). Modulo,
    /// bitwise, and shift operators are always unchecked.
    ///
    /// # Panics
    ///
    /// Panics if called on a comparison or assignment operator.
    pub fn sol_operation_name(self, checked: bool) -> &'static str {
        match self {
            Self::Add | Self::Increment if checked => solx_mlir::Builder::SOL_CADD,
            Self::Add | Self::Increment => solx_mlir::Builder::SOL_ADD,
            Self::Subtract | Self::Decrement if checked => solx_mlir::Builder::SOL_CSUB,
            Self::Subtract | Self::Decrement => solx_mlir::Builder::SOL_SUB,
            Self::Multiply if checked => solx_mlir::Builder::SOL_CMUL,
            Self::Multiply => solx_mlir::Builder::SOL_MUL,
            Self::Divide if checked => solx_mlir::Builder::SOL_CDIV,
            Self::Divide => solx_mlir::Builder::SOL_DIV,
            Self::Remainder => solx_mlir::Builder::SOL_MOD,
            Self::BitwiseAnd => solx_mlir::Builder::SOL_AND,
            Self::BitwiseOr => solx_mlir::Builder::SOL_OR,
            Self::BitwiseXor => solx_mlir::Builder::SOL_XOR,
            Self::ShiftLeft => solx_mlir::Builder::SOL_SHL,
            Self::ShiftRight => solx_mlir::Builder::SOL_SHR,
            _ => unreachable!("sol_operation_name called on non-arithmetic operator: {self:?}"),
        }
    }

    /// Returns the Sol dialect comparison predicate.
    ///
    /// # Panics
    ///
    /// Panics if called on a non-comparison operator.
    pub fn cmp_predicate(self) -> CmpPredicate {
        match self {
            Self::Equal => CmpPredicate::Eq,
            Self::NotEqual => CmpPredicate::Ne,
            Self::Greater => CmpPredicate::Gt,
            Self::GreaterEqual => CmpPredicate::Ge,
            Self::Less => CmpPredicate::Lt,
            Self::LessEqual => CmpPredicate::Le,
            _ => unreachable!("cmp_predicate called on non-comparison operator: {self:?}"),
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
