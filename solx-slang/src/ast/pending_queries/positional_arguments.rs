//!
//! TODO: pure-Slang query pending a home (Slang dev-solx vs solx vs fold) —
//! query-sorting pass. Lifted verbatim from `FunctionEmitter::positional_arguments`.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;

/// The positional arguments of a modifier / base invocation's argument list, or
/// `None` when the list is empty. Named (`{...}`) and call-option argument forms
/// are not positional and yield `None`.
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
