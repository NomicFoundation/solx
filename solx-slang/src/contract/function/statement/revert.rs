//!
//! The `revert` statement, plain or with a custom error.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::RevertStatement;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `revert` statement, plain or with a custom error.
    pub fn revert_statement(&mut self, node: &RevertStatement) {
        let error = match node.error().resolve_to_definition() {
            None => {
                self.current_block().revert("", &[], self);
                return;
            }
            Some(Definition::Error(error)) => error,
            Some(_) => unreachable!("revert target resolves to an error definition"),
        };
        let signature = error
            .compute_canonical_signature()
            .expect("canonical signature is computable for a custom error");
        let values: Vec<_> = self
            .arguments_declaration(&node.arguments(), &error.parameters())
            .into_iter()
            .map(|(_, value)| value)
            .collect();
        self.current_block()
            .revert_custom(&signature, &values, self);
    }
}
