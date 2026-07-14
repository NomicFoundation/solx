//!
//! The `revert` statement, plain or with a custom error.
//!

use slang_solidity_v2::ast::Definition;

use crate::contract::function::expression::call::arguments_declaration::ArgumentsDeclaration;

codegen!(
    /// The `revert` statement, plain or with a custom error.
    RevertStatement -> Effect |node, scope| {
        let error = match node.error().resolve_to_definition() {
            None => {
                scope.current_block().revert("", &[], scope);
                return;
            }
            Some(Definition::Error(error)) => error,
            Some(_) => unreachable!("revert target resolves to an error definition"),
        };
        let signature = error
            .compute_canonical_signature()
            .expect("canonical signature is computable for a custom error");
        let values: Vec<_> =
            ArgumentsDeclaration::emit_ordered(&node.arguments(), &error.parameters(), scope)
                .into_iter()
                .map(|(_, value)| value)
                .collect();
        scope
            .current_block()
            .revert_custom(&signature, &values, scope);
    }
);
