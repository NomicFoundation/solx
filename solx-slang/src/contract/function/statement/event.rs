//!
//! The `emit` event statement.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::EmitStatement;

use solx_mlir::Value;

use crate::scope::function::FunctionScope;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// The `emit` event statement. As in legacy, the indexed arguments run from the last to the
    /// first, each converted or hashed into its topic before the next, and the others follow in
    /// order.
    pub fn emit_statement(&mut self, node: &EmitStatement) {
        let Some(Definition::Event(event)) = node.event().resolve_to_definition() else {
            unreachable!("emit target resolves to an event definition");
        };
        let parameters = event.parameters();
        let arguments: Vec<_> = parameters
            .iter()
            .zip(Self::ordered_arguments(&node.arguments(), &parameters))
            .collect();
        let mut indexed = Vec::new();
        for (parameter, argument) in arguments.iter().rev() {
            if !parameter.is_indexed() {
                continue;
            }
            let parameter_type = self.typing(parameter.get_type());
            indexed.push(if parameter_type.is_scalar() {
                self.converted(argument, parameter_type)
            } else {
                Value::keccak256(self.expression(argument), self)
            });
        }
        indexed.reverse();
        let mut non_indexed = Vec::new();
        for (parameter, argument) in &arguments {
            if parameter.is_indexed() {
                continue;
            }
            let parameter_type = self.typing(parameter.get_type());
            non_indexed.push(self.external_argument(argument, parameter_type));
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
