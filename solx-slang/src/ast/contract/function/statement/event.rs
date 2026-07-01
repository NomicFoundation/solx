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
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::EmitStatement as EmitStatementNode;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameters;
use solx_mlir::ods::sol::EmitOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_statement::EmitStatement;

statement_emit!(EmitStatementNode; |node, context, block| {
    let Some(Definition::Event(event_definition)) = node.event().resolve_to_definition() else {
        unreachable!("emit target resolves to an event definition");
    };
    let parameters = event_definition.parameters();
    let ordered_arguments = match &node.arguments() {
        ArgumentsDeclaration::PositionalArguments(positional) => {
            positional.iter().collect::<Vec<_>>()
        }
        ArgumentsDeclaration::NamedArguments(named) => {
            StatementContext::order_named_event_arguments(named, &parameters)
        }
    };

    let expression_context = context.expression_context();
    let mut indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
    let mut non_indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
    let mut current_block = block;
    for (parameter, argument) in parameters.iter().zip(ordered_arguments) {
        let BlockAnd {
            value,
            block: next_block,
        } = argument.emit(&expression_context, current_block);
        current_block = next_block;
        let indexed = parameter.is_indexed();
        let parameter_type = TypeConversion::resolve_slang_type(
            &parameter
                .get_type()
                .expect("parameter type resolved by semantic analysis"),
            None,
            context.state,
        );
        let value = TypeConversion::from_target_type(parameter_type, context.state).emit(
            value,
            context.state,
            &current_block,
        );
        if indexed {
            // TODO: indexed reference-type parameters (string, bytes,
            // arrays, structs) must store the keccak256 hash of their
            // encoded value as the topic, not the value itself. That
            // lowering is not supported by solc-MLIR yet.
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
                .expect("canonical signature is computable for a named event"),
        )
    };
    context.append_sol_emit(
        signature.as_deref(),
        &indexed_arguments,
        &non_indexed_arguments,
        &current_block,
    );
    Some(current_block)
});

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
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
        let combined_arguments: Vec<Value<'context, 'block>> = indexed_arguments
            .iter()
            .chain(non_indexed_arguments.iter())
            .copied()
            .collect();
        let indexed_count = i8::try_from(indexed_arguments.len())
            .expect("EVM events have at most four indexed arguments");
        let indexed_count_attribute = IntegerAttribute::new(
            Type::from(IntegerType::new(self.state.mlir_context, 8)),
            indexed_count.into(),
        );
        let mut emit_builder =
            EmitOperation::builder(self.state.mlir_context, self.state.location())
                .args(&combined_arguments)
                .indexed_args_count(indexed_count_attribute);
        if let Some(signature) = signature {
            emit_builder =
                emit_builder.signature(StringAttribute::new(self.state.mlir_context, signature));
        }
        block.append_operation(emit_builder.build().into());
    }

    /// Orders named event arguments by the event's parameter declaration order.
    fn order_named_event_arguments(
        named_arguments: &NamedArguments,
        event_parameters: &Parameters,
    ) -> Vec<Expression> {
        let mut arguments = HashMap::new();
        for argument in named_arguments.iter() {
            match arguments.entry(argument.name().name()) {
                Entry::Vacant(entry) => {
                    entry.insert(argument.value());
                }
                Entry::Occupied(entry) => {
                    unreachable!("slang rejects a duplicate named event argument `{}`", entry.key());
                }
            }
        }

        let mut ordered_arguments = Vec::new();
        for parameter in event_parameters.iter() {
            let parameter_name = parameter
                .name()
                .expect("a named-argument event has named parameters")
                .name();
            let argument = arguments
                .remove(&parameter_name)
                .expect("slang matches every named event argument to a parameter");
            ordered_arguments.push(argument);
        }

        ordered_arguments
    }
}
