//!
//! The `emit` event statement.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::EmitStatement;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `emit` event statement. Indexed reference-typed parameters are not yet hashed into their
    /// topics.
    pub fn emit_statement(&mut self, node: &EmitStatement) {
        let Some(Definition::Event(event)) = node.event().resolve_to_definition() else {
            unreachable!("emit target resolves to an event definition");
        };
        let mut indexed = Vec::new();
        let mut non_indexed = Vec::new();
        for (parameter, value) in self.arguments_declaration(&node.arguments(), &event.parameters())
        {
            (if parameter.is_indexed() {
                &mut indexed
            } else {
                &mut non_indexed
            })
            .push(value);
        }

        let signature = (!event.is_anonymous()).then(|| {
            event
                .compute_canonical_signature()
                .expect("canonical signature is computable for a named event")
        });
        self.current_block()
            .emit(signature.as_deref(), &indexed, &non_indexed, self);
    }
}
