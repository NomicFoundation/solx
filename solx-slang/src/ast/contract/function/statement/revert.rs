//!
//! Revert statement emission.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::RevertStatement;

use solx_mlir::LocationPolicy;
use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::RevertOperation;

use crate::ast::analysis::query::parameter_node_ids::ParameterNodeIds;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_statement::EmitStatement;

statement_emit!(RevertStatement; |node, context, block| {
    let error = match node.error().resolve_to_definition() {
        None => {
            let state = context.state;
            mlir_op_void!(
                state,
                &block,
                RevertOperation
                    .signature(StringAttribute::new(state.mlir_context, ""))
                    .args(&[])
            );
            return Some(block);
        }
        Some(Definition::Error(error)) => error,
        Some(_) => unreachable!("slang resolves a revert target to an error definition"),
    };
    let signature = error.compute_canonical_signature().expect("slang validated");
    let parameter_ids = error.parameters().node_ids();
    let parameter_types = AstType::resolve_parameters(
        &error.parameters(),
        LocationPolicy::Declared(None),
        context.state,
    );
    let arguments = CallArguments::for_parameter_ids(&node.arguments(), &parameter_ids);
    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: values,
        block,
    } = arguments.emit_as(&parameter_types, &emitter, block);
    let state = context.state;
    mlir_op_void!(
        state,
        &block,
        RevertOperation
            .signature(StringAttribute::new(state.mlir_context, &signature))
            .args(&values)
            .call(Attribute::unit(state.mlir_context))
    );
    Some(block)
});
