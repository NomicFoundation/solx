//!
//! The `emit` event statement.
//!

use slang_solidity_v2::ast::Definition;

use crate::contract::function::expression::call::arguments_declaration::ArgumentsDeclaration;

codegen!(
    /// The `emit` event statement. Indexed reference-typed parameters are not yet hashed into
    /// their topics.
    EmitStatement -> Effect |node, scope| {
        let Some(Definition::Event(event)) = node.event().resolve_to_definition() else {
            unreachable!("emit target resolves to an event definition");
        };
        let mut indexed = Vec::new();
        let mut non_indexed = Vec::new();
        for (parameter, value) in
            ArgumentsDeclaration::emit_ordered(&node.arguments(), &event.parameters(), scope)
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
        scope
            .current_block()
            .emit(signature.as_deref(), &indexed, &non_indexed, scope);
    }
);
