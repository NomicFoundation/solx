//!
//! Revert statement emission.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::RevertStatement;
use solx_mlir::ods::sol::RevertOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::function::statement::StatementContext;

statement_emit!(RevertStatement; |node, context, block| {
    let error = match node.error().resolve_to_definition() {
        None => {
            let builder = &context.state.builder;
            mlir_op_void!(
                builder,
                &block,
                RevertOperation
                    .signature(StringAttribute::new(builder.context, ""))
                    .args(&[])
            );
            return Some(block);
        }
        Some(Definition::Error(error)) => error,
        Some(_) => unreachable!("slang resolves a revert target to an error definition"),
    };
    let signature = error
        .compute_canonical_signature()
        .expect("slang validated");
    let parameters = error.parameters();
    let parameter_ids = parameters
        .iter()
        .map(|parameter| parameter.node_id())
        .collect::<Vec<_>>();
    let arguments = CallArguments::for_parameter_ids(&node.arguments(), &parameter_ids);
    let parameter_types: Vec<_> = parameters
        .iter()
        .map(|parameter| {
            AstType::resolve(
                &parameter
                    .get_type()
                    .expect("slang validated"),
                LocationPolicy::Declared(None),
                &context.state.builder,
            )
        })
        .collect();
    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: values,
        block,
    } = arguments.emit_as(&parameter_types, &emitter, block);
    let builder = &context.state.builder;
    mlir_op_void!(
        builder,
        &block,
        RevertOperation
            .signature(StringAttribute::new(builder.context, &signature))
            .args(&values)
            .call(Attribute::unit(builder.context))
    );
    Some(block)
});
