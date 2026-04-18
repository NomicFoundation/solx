//!
//! Solidity operator parsed from source text.
//!

use std::str::FromStr;

use melior::ir::Location;
use melior::ir::Value;
use melior::ir::operation::Operation;

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::AddOperation;
use solx_mlir::ods::sol::AndOperation;
use solx_mlir::ods::sol::CAddOperation;
use solx_mlir::ods::sol::CDivOperation;
use solx_mlir::ods::sol::CMulOperation;
use solx_mlir::ods::sol::CSubOperation;
use solx_mlir::ods::sol::DivOperation;
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

    /// Builds a Sol dialect binary operation via ODS-generated builders.
    ///
    /// When `checked` is true, uses checked variants (`sol.cadd`, `sol.csub`,
    /// `sol.cmul`, `sol.cdiv`) for arithmetic operators. Modulo, bitwise,
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
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            Self::ShiftRight => ShrOperation::builder(context, location)
                .lhs(lhs)
                .rhs(rhs)
                .build()
                .into(),
            _ => unreachable!(
                "emit_sol_binary_operation called on non-arithmetic operator: {self:?}"
            ),
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
