//!
//! Sol dialect comparison predicate values.
//!

use melior::ir::attribute::IntegerAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::EqualityExpressionOperator;
use slang_solidity_v2::ast::InequalityExpressionOperator;

use solx_utils::BIT_LENGTH_X64;

/// Sol dialect `sol.cmp` predicate values.
///
/// Signedness is carried by the operand type (`ui256` vs `si256`),
/// not the predicate. Numeric values match the Sol MLIR dialect
/// `CmpPredicate` encoding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum CmpPredicate {
    /// Equal.
    Eq = 0,
    /// Not equal.
    Ne = 1,
    /// Less than.
    Lt = 2,
    /// Less than or equal.
    Le = 3,
    /// Greater than.
    Gt = 4,
    /// Greater than or equal.
    Ge = 5,
}

impl From<EqualityExpressionOperator> for CmpPredicate {
    fn from(operator: EqualityExpressionOperator) -> Self {
        match operator {
            EqualityExpressionOperator::EqualEqual(_) => Self::Eq,
            EqualityExpressionOperator::BangEqual(_) => Self::Ne,
        }
    }
}

impl From<InequalityExpressionOperator> for CmpPredicate {
    fn from(operator: InequalityExpressionOperator) -> Self {
        match operator {
            InequalityExpressionOperator::LessThan(_) => Self::Lt,
            InequalityExpressionOperator::LessThanEqual(_) => Self::Le,
            InequalityExpressionOperator::GreaterThan(_) => Self::Gt,
            InequalityExpressionOperator::GreaterThanEqual(_) => Self::Ge,
        }
    }
}

impl CmpPredicate {
    /// This predicate's encoding as the `i64` [`IntegerAttribute`] the `sol.cmp`
    /// predicate operand demands.
    pub fn attribute(self, context: &melior::Context) -> IntegerAttribute<'_> {
        IntegerAttribute::new(
            IntegerType::new(context, BIT_LENGTH_X64 as u32).into(),
            self as i64,
        )
    }
}
