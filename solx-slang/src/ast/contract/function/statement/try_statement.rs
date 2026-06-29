//!
//! `try` statement emission.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Value;
use melior::ir::operation::OperationLike;
use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::TryStatement;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::TryFallbackKind;
use solx_mlir::ods::sol::TryOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::EmitStatement;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::try_external_call::TryExternalCall;
use crate::ast::contract::function::expression::call::try_function_pointer_call::TryFunctionPointerCall;
use crate::ast::contract::function::expression::call::try_new_expression::TryNewExpression;
use crate::ast::contract::function::statement::StatementContext;

statement_emit!(CatchClause; |node, context, block| {
    let region = block.parent_region().expect("block belongs to a region");
    context.region_pointer = &*region as *const _;
    if let Some(error) = node.error()
        && let Some(parameter) = error.parameters().iter().next()
    {
        let decoded: Value<'context, 'block> = block
            .argument(0)
            .expect("argument index is within the block signature")
            .into();
        context.environment.bind_parameter(
            parameter.node_id(),
            AstType::parameter(parameter.get_type().as_ref(), context.state),
            decoded,
            context.state,
            &block,
        );
    }
    node.body().emit(context, block)
});

statement_emit!(TryStatement; |node, context, block| {
    let expression = node.expression();

    let (status, results, current_block) = {
        let emitter = ExpressionContext::from(&*context);
        if let Some(call) = TryExternalCall::from_expression(&expression) {
            call.emit(&emitter, block)
        } else if let Some(new) = TryNewExpression::from_expression(&expression) {
            new.emit(&emitter, block)
        } else if let Some(call) = TryFunctionPointerCall::from_expression(&expression) {
            call.emit(&emitter, block)
        } else {
            unreachable!(
                "a try expression is an external call, an external function-pointer call, or a contract creation"
            )
        }
    };

    let mut panic_clause: Option<CatchClause> = None;
    let mut error_clause: Option<CatchClause> = None;
    let mut fallback_clause: Option<CatchClause> = None;
    let mut fallback_kind = TryFallbackKind::None;
    for clause in node.catch_clauses().iter() {
        match clause.error() {
            None => {
                fallback_kind = TryFallbackKind::Empty;
                fallback_clause = Some(clause);
            }
            Some(error) if error.name().is_none() => {
                fallback_kind = TryFallbackKind::Bytes;
                fallback_clause = Some(clause);
            }
            Some(error) => {
                let parameter = error
                    .parameters()
                    .iter()
                    .next()
                    .expect("slang validated");
                match parameter
                    .get_type()
                    .expect("slang validated")
                {
                    SlangType::String(_) => error_clause = Some(clause),
                    SlangType::Integer(_) => panic_clause = Some(clause),
                    _ => unreachable!("a typed catch clause binds Error(string) or Panic(uint256)"),
                }
            }
        }
    }

    let saved_region = context.region_pointer;
    let state = context.state;
    let has_panic = panic_clause.is_some();
    let has_error = error_clause.is_some();
    let success_region = Region::new();
    success_region.append_block(Block::new(&[]));
    let panic_region = Region::new();
    if has_panic {
        panic_region.append_block(Block::new(&[(
            AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
            state.location(),
        )]));
    }
    let error_region = Region::new();
    if has_error {
        error_region.append_block(Block::new(&[(
            AstType::string(state.mlir_context, solx_utils::DataLocation::Memory).into_mlir(),
            state.location(),
        )]));
    }
    let fallback_region = Region::new();
    match fallback_kind {
        TryFallbackKind::None => {}
        TryFallbackKind::Empty => {
            fallback_region.append_block(Block::new(&[]));
        }
        TryFallbackKind::Bytes => {
            fallback_region.append_block(Block::new(&[(
                AstType::string(state.mlir_context, solx_utils::DataLocation::Memory)
                    .into_mlir(),
                state.location(),
            )]));
        }
    }
    let operation = current_block.append_operation(mlir_op_build!(
        state,
        TryOperation
            .status(status)
            .success_region(success_region)
            .panic_region(panic_region)
            .error_region(error_region)
            .fallback_region(fallback_region)
    ));
    let success_block = operation
        .region(0)
        .expect("sol.try has a success region")
        .first_block()
        .expect("success region has a block");
    let panic_block = has_panic.then(|| {
        operation
            .region(1)
            .expect("sol.try has a panic region")
            .first_block()
            .expect("panic region has a block")
    });
    let error_block = has_error.then(|| {
        operation
            .region(2)
            .expect("sol.try has an error region")
            .first_block()
            .expect("error region has a block")
    });
    let fallback_block = (!matches!(fallback_kind, TryFallbackKind::None)).then(|| {
        operation
            .region(3)
            .expect("sol.try has a fallback region")
            .first_block()
            .expect("fallback region has a block")
    });

    let success_region = success_block
        .parent_region()
        .expect("block belongs to a region");
    context.region_pointer = &*success_region as *const _;
    if let Some(parameters) = node.returns() {
        for (parameter, result) in parameters.iter().zip(results.iter()) {
            if parameter.name().is_none() {
                continue;
            }
            context.environment.bind_parameter(
                parameter.node_id(),
                AstType::parameter(parameter.get_type().as_ref(), context.state),
                *result,
                context.state,
                &success_block,
            );
        }
    }
    let success_end = node.body().emit(context, success_block);
    if let Some(end) = success_end {
        mlir_op_void!(context.state, &end, YieldOperation.ins(&[]));
    }

    for (catch_block, clause) in [
        (panic_block, panic_clause),
        (error_block, error_clause),
        (fallback_block, fallback_clause),
    ] {
        if let Some(catch_block) = catch_block {
            let clause = clause.expect("a populated catch region implies its clause");
            if let Some(end) = clause.emit(context, catch_block) {
                mlir_op_void!(context.state, &end, YieldOperation.ins(&[]));
            }
        }
    }

    context.region_pointer = saved_region;
    Some(current_block)
});
