//! Event emit statement lowering.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::EmitStatement;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameters;
use solx_mlir::Context;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context> StatementEmitter<'state, 'context> {
    /// Lowers an `emit Event(args);` statement to a `sol.emit` operation.
    ///
    /// Resolves the event definition, classifies each argument as indexed
    /// or non-indexed per the event's parameter declaration, evaluates
    /// argument expressions in declaration order, and emits the op with
    /// the canonical signature (or `None` for anonymous events).
    ///
    /// # Errors
    ///
    /// Returns an error if the emit target does not resolve to an event,
    /// the canonical signature cannot be computed, the argument count does
    /// not match the event's parameter count, named arguments cannot be
    /// matched to parameters, or any argument expression cannot be lowered.
    pub fn emit_event(
        &self,
        emit_statement: &EmitStatement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let Some(Definition::Event(event_definition)) =
            emit_statement.event().resolve_to_definition()
        else {
            anyhow::bail!("emit target does not resolve to an event definition");
        };
        let parameters = event_definition.parameters();
        let ordered_arguments = match &emit_statement.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                positional.iter().collect::<Vec<_>>()
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                Self::order_named_event_arguments(named, &parameters)?
            }
        };
        anyhow::ensure!(
            ordered_arguments.len() == parameters.len(),
            "event argument count {} does not match parameter count {}",
            ordered_arguments.len(),
            parameters.len()
        );

        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
        let mut indexed_arguments: Vec<Value<'context>> = Vec::new();
        let mut non_indexed_arguments: Vec<Value<'context>> = Vec::new();
        for (parameter, argument) in parameters.iter().zip(ordered_arguments) {
            let value = emitter.emit_value(&argument, context)?;
            let indexed = parameter.is_indexed();
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                context,
            );
            let value =
                TypeConversion::from_target_type(parameter_type, context).emit(value, context);
            if indexed {
                // TODO: indexed reference-type parameters must store the
                // keccak256 hash of their encoded value as the topic, not the
                // value itself. That lowering is not supported yet.
                indexed_arguments.push(value);
            } else {
                non_indexed_arguments.push(value);
            }
        }

        let signature = if event_definition.is_anonymous() {
            None
        } else {
            Some(
                event_definition
                    .compute_canonical_signature()
                    .ok_or_else(|| {
                        anyhow::anyhow!("cannot compute canonical signature for event")
                    })?,
            )
        };
        let block = context.current_block();
        block.emit(
            signature.as_deref(),
            &indexed_arguments,
            &non_indexed_arguments,
            context,
        );
        Ok(())
    }

    /// Orders named event arguments by the event's parameter declaration order.
    fn order_named_event_arguments(
        named_arguments: &NamedArguments,
        event_parameters: &Parameters,
    ) -> anyhow::Result<Vec<Expression>> {
        let mut arguments = HashMap::new();
        for argument in named_arguments.iter() {
            match arguments.entry(argument.name().name()) {
                Entry::Vacant(entry) => {
                    entry.insert(argument.value());
                }
                Entry::Occupied(entry) => {
                    anyhow::bail!("duplicate named event argument `{}`", entry.key());
                }
            }
        }

        let mut ordered_arguments = Vec::new();
        for parameter in event_parameters.iter() {
            let parameter_name = parameter
                .name()
                .ok_or_else(|| {
                    anyhow::anyhow!("cannot match named event argument to unnamed event parameter")
                })?
                .name();
            let argument = arguments.remove(&parameter_name).ok_or_else(|| {
                anyhow::anyhow!("missing named event argument `{parameter_name}`")
            })?;
            ordered_arguments.push(argument);
        }

        anyhow::ensure!(
            arguments.is_empty(),
            "unknown named event argument(s): {}",
            arguments
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(ordered_arguments)
    }
}
