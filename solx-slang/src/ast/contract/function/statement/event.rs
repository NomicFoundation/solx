//! Event emit statement lowering.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::EmitStatement;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::NamedArguments;
use slang_solidity::backend::ir::ast::Parameters;
use solx_mlir::ods::sol::EmitOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
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

        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let mut indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
        let mut non_indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
        let mut current_block = block;
        for (parameter, argument) in parameters.iter().zip(ordered_arguments) {
            let (value, next_block) = emitter.emit_value(&argument, current_block)?;
            current_block = next_block;
            if parameter.indexed() {
                // TODO: indexed reference-type parameters (string, bytes,
                // arrays, structs) must store the keccak256 hash of their
                // encoded value as the topic, not the value itself. That
                // lowering is not supported by solc-MLIR yet.
                indexed_arguments.push(value);
            } else {
                non_indexed_arguments.push(value);
            }
        }

        let signature = if event_definition.anonymous_keyword() {
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
        self.append_sol_emit(
            signature.as_deref(),
            &indexed_arguments,
            &non_indexed_arguments,
            &current_block,
        );
        Ok(Some(current_block))
    }

    /// Appends a `sol.emit` operation with the given indexed and non-indexed
    /// arguments. EVM events have at most four indexed topics, so the count
    /// always fits in the dialect's `i8` `indexedArgsCount` attribute.
    fn append_sol_emit(
        &self,
        signature: Option<&str>,
        indexed_arguments: &[Value<'context, 'block>],
        non_indexed_arguments: &[Value<'context, 'block>],
        block: &BlockRef<'context, 'block>,
    ) {
        let builder = &self.state.builder;
        let combined_arguments: Vec<Value<'context, 'block>> = indexed_arguments
            .iter()
            .chain(non_indexed_arguments.iter())
            .copied()
            .collect();
        let indexed_count = i8::try_from(indexed_arguments.len())
            .expect("EVM events have at most four indexed arguments");
        let indexed_count_attribute = IntegerAttribute::new(
            Type::from(IntegerType::new(builder.context, 8)),
            indexed_count.into(),
        );
        let mut emit_builder = EmitOperation::builder(builder.context, builder.unknown_location)
            .args(&combined_arguments)
            .indexed_args_count(indexed_count_attribute);
        if let Some(signature) = signature {
            emit_builder = emit_builder.signature(StringAttribute::new(builder.context, signature));
        }
        block.append_operation(emit_builder.build().into());
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
