//!
//! Event emit statement lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::EmitStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

use super::StatementEmitter;
use super::named_arguments::order_named_arguments;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers `emit Event(args);` to `sol.emit`.
    ///
    /// Each argument is classified as an indexed topic or a non-indexed data
    /// field per the event declaration, coerced to its parameter type, and the
    /// op is tagged with the event's canonical signature (`None` for an
    /// anonymous event).
    pub(super) fn emit_event(
        &self,
        emit_statement: &EmitStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(Definition::Event(event)) = emit_statement.event().resolve_to_definition() else {
            unreachable!("an emit target resolves to an event definition");
        };
        let parameters = event.parameters();
        let ordered_arguments = match emit_statement.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                positional.iter().collect::<Vec<_>>()
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                order_named_arguments(&named, &parameters)
            }
        };

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let mut indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
        let mut non_indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
        let mut block = block;
        for (parameter, argument) in parameters.iter().zip(ordered_arguments) {
            let (value, next_block) = emitter.emit_value(&argument, block)?;
            block = next_block;
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("the binder types every event parameter"),
                None,
                &self.state.builder,
            );
            let value = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                value,
                &self.state.builder,
                &block,
            );
            // A reference-typed indexed parameter (string / bytes / array /
            // struct) should hash its encoded value into the topic; that
            // lowering is not yet wired, so the value is taken as-is.
            if parameter.indexed().is_some() {
                indexed_arguments.push(value);
            } else {
                non_indexed_arguments.push(value);
            }
        }

        let signature = if event.anonymous_keyword().is_some() {
            None
        } else {
            Some(
                event
                    .compute_canonical_signature()
                    .expect("the binder computes a canonical signature for a named event"),
            )
        };
        self.state.builder.emit_sol_emit(
            signature.as_deref(),
            &indexed_arguments,
            &non_indexed_arguments,
            &block,
        );
        Ok(Some(block))
    }
}
