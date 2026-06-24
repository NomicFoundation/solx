//!
//! Positional-arguments query for an argument list (pure-Slang).
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;

/// The positional arguments of an argument list, or `None` when empty or non-positional (named / call-option).
pub trait PositionalArguments {
    /// This argument list's positional expressions, or `None` when there are none.
    fn positional_arguments(&self) -> Option<Vec<Expression>>;
}

impl PositionalArguments for ArgumentsDeclaration {
    fn positional_arguments(&self) -> Option<Vec<Expression>> {
        match self {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                let expressions: Vec<Expression> = positional.iter().collect();
                (!expressions.is_empty()).then_some(expressions)
            }
            _ => None,
        }
    }
}
