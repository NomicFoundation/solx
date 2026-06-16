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
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::TryStatement;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::TryFallbackKind;
use solx_mlir::ods::sol::TryOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::try_external_call::TryExternalCall;
use crate::ast::contract::function::statement::StatementContext;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Casts a decoded value to the bound parameter's declared type, stores it
    /// into a fresh stack slot, and defines the parameter in scope by its node id
    /// — the shared binding for the success `returns (...)`, the typed
    /// `Error`/`Panic` reason/code, and the low-level `bytes` data.
    fn bind_parameter(
        &mut self,
        parameter: &Parameter,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let parameter_type = parameter
            .get_type()
            .map(|slang_type| {
                crate::ast::Type::resolve(
                    &slang_type,
                    LocationPolicy::Declared(None),
                    &self.state.builder,
                )
            })
            .unwrap_or_else(|| {
                crate::ast::Type::unsigned(self.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir()
            });
        let cast = crate::ast::Value::from(value).cast(
            crate::ast::Type::new(parameter_type),
            &self.state.builder,
            block,
        );
        let pointer = crate::ast::Pointer::stack_slot(
            crate::ast::Type::new(parameter_type),
            &self.state.builder,
            block,
        );
        pointer.store(cast, &self.state.builder, block);
        self.environment
            .define_variable(parameter.node_id(), pointer.into_mlir());
    }
}

// One catch clause's region body. The `sol.try` conversion delivers the decoded
// panic code / error reason / raw returndata as block argument 0, bound to the
// clause's parameter (a parameter-less `catch {}` binds nothing) before the body
// runs. The caller terminates the region with the trailing `sol.yield`.
statement_emit!(CatchClause; |node, context, block| {
    let region = block.parent_region().expect("block belongs to a region");
    context.set_region(&region);
    if let Some(error) = node.error()
        && let Some(parameter) = error.parameters().iter().next()
    {
        let decoded: Value<'context, 'block> = block
            .argument(0)
            .expect("argument index is within the block signature")
            .into();
        context.bind_parameter(&parameter, decoded, &block);
    }
    context.emit_block(node.body().statements(), block)
});

// A `try` statement lowers to a `sol.try`: an external call with try semantics
// yields the success `status`, and the op carries four regions — success, panic,
// error, fallback. The success region binds the declared `returns (...)` and runs
// the body; each present catch clause populates its region from the decoded
// payload. Absent clauses leave empty regions, and the op's conversion owns the
// selector dispatch, payload decode, and raw re-revert when no clause matches.
statement_emit!(TryStatement; |node, context, block| {
    let expression = node.expression();

    // Only a lowerable external call (`recv.f(args)` / `recv.f{value: v}(args)`)
    // carries a real catch path; any other `try` expression runs only the success
    // body, binding the first declared (named) return.
    let Some(try_call) = TryExternalCall::classify(&expression) else {
        let BlockAnd { value, block: current_block } = {
            let emitter = ExpressionContext::from(&*context);
            expression.emit(&emitter, block)
        };
        if let Some(parameters) = node.returns()
            && let Some(parameter) = parameters.iter().next()
            && parameter.name().is_some()
        {
            context.bind_parameter(&parameter, value.into_mlir(), &current_block);
        }
        return context.emit_block(node.body().statements(), current_block);
    };

    let (status, results, current_block) =
        try_call.emit(&ExpressionContext::from(&*context), block);

    // Classify the catch clauses into the `sol.try` regions: a parameter-less
    // `catch {}` has no error group; a low-level `catch (bytes r)` an unnamed one;
    // a typed `catch Error(...)` / `catch Panic(...)` is told apart by its bound
    // parameter's type (`Error(string)` vs `Panic(uint256)`), never by identifier
    // text.
    let mut panic_clause: Option<CatchClause> = None;
    let mut error_clause: Option<CatchClause> = None;
    let mut fallback_clause: Option<CatchClause> = None;
    let mut fallback_kind = TryFallbackKind::None;
    for clause in node.catch_clauses().iter() {
        match clause.error() {
            None => {
                fallback_kind = TryFallbackKind::Parameterless;
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
                    .expect("a typed catch clause declares one parameter");
                match parameter
                    .get_type()
                    .expect("catch parameter type resolved by semantic analysis")
                {
                    SlangType::String(_) => error_clause = Some(clause),
                    SlangType::Integer(_) => panic_clause = Some(clause),
                    _ => unreachable!("a typed catch clause binds Error(string) or Panic(uint256)"),
                }
            }
        }
    }

    let saved_region = context.region_pointer;
    let builder = &context.state.builder;
    let has_panic = panic_clause.is_some();
    let has_error = error_clause.is_some();
    // sol.try carries four regions — success, panic, error, fallback. An absent
    // clause leaves an empty region; a present catch region carries its decoded
    // payload (panic code `ui256`, error reason / bytes `string`) as block
    // argument 0.
    let success_region = Region::new();
    success_region.append_block(Block::new(&[]));
    let panic_region = Region::new();
    if has_panic {
        panic_region.append_block(Block::new(&[(
            crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
            builder.unknown_location,
        )]));
    }
    let error_region = Region::new();
    if has_error {
        error_region.append_block(Block::new(&[(
            crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory).into_mlir(),
            builder.unknown_location,
        )]));
    }
    let fallback_region = Region::new();
    match fallback_kind {
        TryFallbackKind::None => {}
        TryFallbackKind::Parameterless => {
            fallback_region.append_block(Block::new(&[]));
        }
        TryFallbackKind::Bytes => {
            fallback_region.append_block(Block::new(&[(
                crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory)
                    .into_mlir(),
                builder.unknown_location,
            )]));
        }
    }
    let operation = current_block.append_operation(sol_op_build!(
        builder,
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

    // Success region: bind the declared returns from the call results, then run
    // the body.
    let success_region = success_block
        .parent_region()
        .expect("block belongs to a region");
    context.set_region(&success_region);
    if let Some(parameters) = node.returns() {
        for (parameter, result) in parameters.iter().zip(results.iter()) {
            if parameter.name().is_none() {
                continue;
            }
            context.bind_parameter(&parameter, *result, &success_block);
        }
    }
    let success_end = context.emit_block(node.body().statements(), success_block);
    if let Some(end) = success_end {
        sol_op_void!(&context.state.builder, &end, YieldOperation.ins(&[]));
    }

    // Each present catch region: bind its decoded payload and run its body, then
    // terminate with `sol.yield`. Emission order (panic, error, fallback) is fixed.
    for (catch_block, clause) in [
        (panic_block, panic_clause),
        (error_block, error_clause),
        (fallback_block, fallback_clause),
    ] {
        if let Some(catch_block) = catch_block {
            let clause = clause.expect("a populated catch region implies its clause");
            if let Some(end) = clause.emit(context, catch_block) {
                sol_op_void!(&context.state.builder, &end, YieldOperation.ins(&[]));
            }
        }
    }

    context.region_pointer = saved_region;
    Some(current_block)
});
