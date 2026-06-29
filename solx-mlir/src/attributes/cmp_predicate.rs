//!
//! Sol dialect comparison predicate values.
//!

use slang_solidity_v2::ast::EqualityExpressionOperator;
use slang_solidity_v2::ast::InequalityExpressionOperator;

/// Sol dialect `sol.cmp` predicate values (signedness is carried by the operand type, not the predicate).
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

sol_predicate_attribute!(CmpPredicate);

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
    /// The user-defined operator this predicate corresponds to (`a == b` on a UDVT with a
    /// `using {f as ==} for T` binding calls `f` instead of emitting `sol.cmp`). Every predicate
    /// maps to exactly one [`UserDefinedOperator`].
    pub fn user_defined_operator(self) -> crate::UserDefinedOperator {
        match self {
            Self::Eq => crate::UserDefinedOperator::Eq,
            Self::Ne => crate::UserDefinedOperator::Ne,
            Self::Lt => crate::UserDefinedOperator::Lt,
            Self::Le => crate::UserDefinedOperator::Le,
            Self::Gt => crate::UserDefinedOperator::Gt,
            Self::Ge => crate::UserDefinedOperator::Ge,
        }
    }
}
