//!
//! Named call-argument ordering.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameters;

/// Orders named arguments `{b: y, a: x}` by the callee's parameter declaration
/// order, returning the argument value expressions in that order.
///
/// The binder validates that the names cover the parameters exactly with no
/// duplicates, so a missing or unmatched name is an invariant violation, not a
/// user error.
pub(super) fn order_named_arguments(
    named_arguments: &NamedArguments,
    parameters: &Parameters,
) -> Vec<Expression> {
    let mut by_name: HashMap<String, Expression> = named_arguments
        .iter()
        .map(|argument| (argument.name().name(), argument.value()))
        .collect();
    parameters
        .iter()
        .map(|parameter| {
            let name = parameter
                .name()
                .expect("a named argument matches a named parameter")
                .name();
            by_name
                .remove(&name)
                .expect("the binder supplies a value for every parameter")
        })
        .collect()
}
