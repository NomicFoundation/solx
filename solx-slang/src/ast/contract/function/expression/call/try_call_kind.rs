//!
//! Ahead-of-time classification of the external call a `try` statement guards, so emission is one
//! exhaustive `match` rather than a chain of `Option`-returning probes.
//!

use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::call::external_member_call::ExternalMemberCall;
use crate::ast::contract::function::expression::call::function_pointer_call::FunctionPointerCall;
use crate::ast::contract::function::expression::call::try_call::TryCall;
use crate::ast::contract::function::expression::call::try_new_expression::TryNewExpression;

/// The one kind of external call a `try` statement guards. The variants are mutually exclusive and
/// tested in declaration order.
pub enum TryCallKind {
    /// `try contract.f(...)`: an external call to a named contract function.
    External(TryCall<ExternalMemberCall>),
    /// `try new C(...)`: an external contract creation.
    NewExpression(TryNewExpression),
    /// `try functionPointer(...)`: a call through an external function pointer.
    FunctionPointer(TryCall<FunctionPointerCall>),
}

impl TryCallKind {
    /// Classifies the guarded expression into the single kind that emits it.
    pub fn from_expression(expression: &Expression) -> Self {
        if let Some(inner) = TryCall::<ExternalMemberCall>::from_expression(expression) {
            return Self::External(inner);
        }
        if let Some(inner) = TryNewExpression::from_expression(expression) {
            return Self::NewExpression(inner);
        }
        if let Some(inner) = TryCall::<FunctionPointerCall>::from_expression(expression) {
            return Self::FunctionPointer(inner);
        }
        unreachable!(
            "a try expression is an external call, an external function-pointer call, or a contract creation"
        )
    }
}
