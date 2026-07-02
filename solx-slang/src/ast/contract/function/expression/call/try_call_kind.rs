//!
//! Ahead-of-time classification of the operation a `try` statement guards, so emission is one
//! exhaustive `match` rather than a chain of `Option`-returning probes.
//!

use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::call::try_call::TryCall;
use crate::ast::contract::function::expression::call::try_new_expression::TryNewExpression;

/// The one operation a `try` statement guards. The variants are mutually exclusive and tested in
/// declaration order, so an earlier match wins.
pub enum TryCallKind {
    /// `try c.foo(args)`: an external call to a contract-instance method.
    External(TryCall),
    /// `try new C(args)`: a contract creation.
    NewExpression(TryNewExpression),
}

impl TryCallKind {
    /// Classifies the guarded expression into the single kind that emits it.
    pub fn from_expression(expression: &Expression) -> Self {
        if let Some(call) = TryCall::from_expression(expression) {
            return Self::External(call);
        }
        if let Some(new) = TryNewExpression::from_expression(expression) {
            return Self::NewExpression(new);
        }
        unreachable!("a try statement guards an external contract method call or a contract creation");
    }
}
