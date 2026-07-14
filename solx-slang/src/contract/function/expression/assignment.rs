//!
//! Assignment expressions, plain and compound, through the left operand's place.
//!

codegen!(
    AssignmentExpression(AssignmentExpressionOperator) -> compound {
        PlusEqual => add(checked),
        MinusEqual => subtract(checked),
        AsteriskEqual => multiply(checked),
        SlashEqual => divide(checked),
        PercentEqual => remainder,
        AmpersandEqual => bitand,
        BarEqual => bitor,
        CaretEqual => bitxor,
        LessThanLessThanEqual => shl,
        GreaterThanGreaterThanEqual | GreaterThanGreaterThanGreaterThanEqual => shr,
    }
);
