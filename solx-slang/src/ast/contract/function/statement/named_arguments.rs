//!
//! Named call-argument ordering, shared by event emission and custom-error
//! reverts.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameters;

/// Orders named arguments to match the callee's parameter declaration order.
///
/// The binder validates that named arguments are unique and cover every
/// parameter, so each lookup is an invariant.
pub fn order_named_arguments(
    named_arguments: &NamedArguments,
    parameters: &Parameters,
) -> Vec<Expression> {
    let mut arguments: HashMap<String, Expression> = named_arguments
        .iter()
        .map(|argument| (argument.name().name(), argument.value()))
        .collect();
    parameters
        .iter()
        .map(|parameter| {
            let parameter_name = parameter
                .name()
                .expect("a named-argument call targets named parameters")
                .name();
            arguments
                .remove(&parameter_name)
                .expect("the binder matches a named argument to every parameter")
        })
        .collect()
}
