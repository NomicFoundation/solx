//!
//! The lexically scoped statement sequence and the braced blocks that carry one.
//!

use slang_solidity_v2::ast::Block;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::UncheckedBlock;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// A statement sequence emitted inside its own lexical scope, stopping after a terminator.
    pub fn statements(&mut self, node: &Statements) {
        self.nested(|scope| {
            for statement in node.iter() {
                scope.statement(&statement);
                if scope.current_block().is_terminated() {
                    break;
                }
            }
        });
    }

    /// A braced block: `{ ... }`.
    pub fn block(&mut self, node: &Block) {
        self.statements(&node.statements());
    }

    /// An `unchecked { ... }` block.
    pub fn unchecked_block(&mut self, node: &UncheckedBlock) {
        self.unchecked(|scope| scope.statements(&node.block().statements()));
    }
}
