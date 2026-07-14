//!
//! The lexically scoped statement sequence and the braced blocks that carry one.
//!

use crate::contract::function::statement::Statement;

codegen!(
    /// A statement sequence emitted inside its own lexical scope, stopping after a terminator.
    Statements -> Effect |node, scope| {
        scope.nested(|scope| {
            for statement in node.iter() {
                Statement::emit(&statement, scope);
                if scope.current_block().is_terminated() {
                    break;
                }
            }
        });
    }

    /// A braced block: `{ ... }`.
    Block -> Effect |node, scope| {
        Statements::emit(&node.statements(), scope);
    }

    /// An `unchecked { ... }` block.
    UncheckedBlock -> Effect |node, scope| {
        scope.unchecked(|scope| Statements::emit(&node.block().statements(), scope));
    }
);
