//!
//! Comparison expressions: equality and inequality over reconciled operand types.
//!

codegen!(
    EqualityExpression(EqualityExpressionOperator) -> compare {
        EqualEqual => Eq,
        BangEqual => Ne,
    }

    InequalityExpression(InequalityExpressionOperator) -> compare {
        LessThan => Lt,
        LessThanEqual => Le,
        GreaterThan => Gt,
        GreaterThanEqual => Ge,
    }
);
