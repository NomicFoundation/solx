//!
//! Arithmetic expressions: additive, multiplicative, exponentiation, bitwise, and shift tables.
//!

codegen!(
    AdditiveExpression(AdditiveExpressionOperator) -> binary {
        Plus => add(checked),
        Minus => subtract(checked),
    }

    MultiplicativeExpression(MultiplicativeExpressionOperator) -> binary {
        Asterisk => multiply(checked),
        Slash => divide(checked),
        Percent => remainder,
    }

    ExponentiationExpression -> binary(exponentiate(checked))
    BitwiseAndExpression -> binary(bitand)
    BitwiseOrExpression -> binary(bitor)
    BitwiseXorExpression -> binary(bitxor)

    ShiftExpression(ShiftExpressionOperator) -> binary {
        LessThanLessThan => shl,
        GreaterThanGreaterThan | GreaterThanGreaterThanGreaterThan => shr,
    }
);
