//!
//! The positional argument list of a call.
//!

use crate::contract::function::expression::Expression;

codegen!(
    /// The positional argument list of a call.
    PositionalArguments -> Values |node, scope| {
        node.iter()
            .map(|argument| Expression::emit(&argument, scope))
            .collect()
    }
);
