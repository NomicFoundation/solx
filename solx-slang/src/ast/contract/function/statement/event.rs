//!
//! Event emit statement emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::EmitStatement as EmitStatementNode;
use solx_mlir::ods::sol::EmitOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::function::statement::StatementContext;

statement_emit!(EmitStatementNode; |node, context, block| {
    let Some(Definition::Event(event_definition)) = node.event().resolve_to_definition() else {
        unreachable!("slang resolves an emit target to an event definition");
    };
    let parameters = event_definition.parameters();
    let parameter_ids = parameters
        .iter()
        .map(|parameter| parameter.node_id())
        .collect::<Vec<_>>();
    let arguments = CallArguments::for_parameter_ids(&node.arguments(), &parameter_ids);

    let emitter = ExpressionContext::from(&*context);
    let mut indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
    let mut non_indexed_arguments: Vec<Value<'context, 'block>> = Vec::new();
    let mut current_block = block;
    for (parameter, argument) in parameters.iter().zip(arguments.expressions.iter()) {
        let BlockAnd {
            value,
            block: next_block,
        } = argument.emit(&emitter, current_block);
        current_block = next_block;
        let indexed = parameter.is_indexed();
        let parameter_type = AstType::resolve(
            &parameter
                .get_type()
                .expect("slang validated"),
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
        let value = value
            .cast(
                AstType::new(parameter_type),
                &context.state.builder,
                &current_block,
            )
            .into_mlir();
        if indexed {
            // TODO: an indexed reference-type parameter must store the keccak256 hash of its
            // encoded value as the topic, not the value.
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
                .expect("slang validated"),
        )
    };
    let builder = &context.state.builder;
    let combined_arguments: Vec<Value<'context, 'block>> = indexed_arguments
        .iter()
        .chain(non_indexed_arguments.iter())
        .copied()
        .collect();
    let indexed_count = i8::try_from(indexed_arguments.len())
        .expect("slang validated");
    let indexed_count_attribute = IntegerAttribute::new(
        Type::from(IntegerType::new(builder.context, 8)),
        indexed_count.into(),
    );
    let mut emit_builder = EmitOperation::builder(builder.context, builder.unknown_location)
        .args(&combined_arguments)
        .indexed_args_count(indexed_count_attribute);
    if let Some(signature) = signature.as_deref() {
        emit_builder = emit_builder.signature(StringAttribute::new(builder.context, signature));
    }
    current_block.append_operation(emit_builder.build().into());
    Some(current_block)
});
