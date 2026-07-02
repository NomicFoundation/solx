//!
//! `try` statement lowering: an external call guarded by success and `catch` clauses.
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

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::TryOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::contract::function::expression::call::try_call_kind::TryCallKind;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_statement::EmitStatement;

/// The shape of a `sol.try` fallback region, selecting how its block is signed and bound.
#[derive(Clone, Copy)]
enum FallbackKind {
    /// No `catch {}` / `catch (bytes)` clause: the region is empty and the op re-reverts raw data.
    None,
    /// Empty `catch { ... }`: the region runs its body with no bound value.
    Empty,
    /// Low-level `catch (bytes memory data) { ... }`: the region binds the returndata as `bytes`.
    Bytes,
}

statement_emit!(TryStatement; |node, context, block| {
    let expression = node.expression();
    let (status, results, current_block) = {
        let expression_context = context.expression_context();
        match TryCallKind::from_expression(&expression) {
            TryCallKind::FunctionPointer(call) => call.emit(&expression_context, block),
            TryCallKind::External(call) => call.emit(&expression_context, block),
            TryCallKind::NewExpression(new) => new.emit(&expression_context, block),
        }
    };

    let mut panic_clause: Option<CatchClause> = None;
    let mut error_clause: Option<CatchClause> = None;
    let mut fallback_clause: Option<CatchClause> = None;
    let mut fallback_kind = FallbackKind::None;
    for clause in node.catch_clauses().iter() {
        match clause.error() {
            None => {
                fallback_kind = FallbackKind::Empty;
                fallback_clause = Some(clause);
            }
            Some(error) if error.name().is_none() => {
                fallback_kind = FallbackKind::Bytes;
                fallback_clause = Some(clause);
            }
            Some(error) => {
                let parameter = error.parameters().iter().next().expect("slang validated");
                match parameter.get_type().expect("slang validated") {
                    SlangType::String(_) => error_clause = Some(clause),
                    SlangType::Integer(_) => panic_clause = Some(clause),
                    _ => unreachable!("a typed catch clause binds Error(string) or Panic(uint256)"),
                }
            }
        }
    }

    let state = context.state;
    let success_region = Region::new();
    success_region.append_block(Block::new(&[]));
    let panic_region = Region::new();
    if panic_clause.is_some() {
        panic_region.append_block(Block::new(&[(
            AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
            state.location(),
        )]));
    }
    let error_region = Region::new();
    if error_clause.is_some() {
        error_region.append_block(Block::new(&[(
            AstType::string(state.mlir_context, solx_utils::DataLocation::Memory).into_mlir(),
            state.location(),
        )]));
    }
    let fallback_region = Region::new();
    match fallback_kind {
        FallbackKind::None => {}
        FallbackKind::Empty => {
            fallback_region.append_block(Block::new(&[]));
        }
        FallbackKind::Bytes => {
            fallback_region.append_block(Block::new(&[(
                AstType::string(state.mlir_context, solx_utils::DataLocation::Memory).into_mlir(),
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
    let success_region = operation.region(0).expect("sol.try has a success region");
    let panic_region = operation.region(1).expect("sol.try has a panic region");
    let error_region = operation.region(2).expect("sol.try has an error region");
    let fallback_region = operation.region(3).expect("sol.try has a fallback region");

    let saved_region = context.region_pointer;

    context.set_region(&success_region);
    let success_block = success_region.first_block().expect("success region has a block");
    context.environment.enter_scope();
    if let Some(parameters) = node.returns() {
        for (parameter, &result) in parameters.iter().zip(results.iter()) {
            context.bind_catch_value(&parameter, result, &success_block);
        }
    }
    if let Some(end) = node.body().emit(context, success_block) {
        mlir_op_void!(state, &end, YieldOperation.ins(&[]));
    }
    context.environment.exit_scope();

    for (region, clause) in [
        (panic_region, panic_clause),
        (error_region, error_clause),
        (fallback_region, fallback_clause),
    ] {
        let Some(clause) = clause else {
            continue;
        };
        context.set_region(&region);
        let catch_block = region.first_block().expect("a populated catch region has a block");
        context.environment.enter_scope();
        if let Some(error) = clause.error()
            && let Some(parameter) = error.parameters().iter().next()
        {
            let argument: Value<'context, 'block> = catch_block
                .argument(0)
                .expect("a typed catch region binds its decoded value")
                .into();
            context.bind_catch_value(&parameter, argument, &catch_block);
        }
        if let Some(end) = clause.body().emit(context, catch_block) {
            mlir_op_void!(state, &end, YieldOperation.ins(&[]));
        }
        context.environment.exit_scope();
    }

    context.region_pointer = saved_region;
    Some(current_block)
});

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Binds a `try` success return or `catch` parameter: stores `value`, coerced to the parameter's
    /// declared type, into a fresh stack slot registered under the parameter's name.
    fn bind_catch_value(
        &mut self,
        parameter: &Parameter,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let Some(identifier) = parameter.name() else {
            return;
        };
        let declared_type = TypeConversion::resolve_slang_type(
            &parameter.get_type().expect("slang types every catch parameter"),
            None,
            self.state,
        );
        let cast = TypeConversion::from_target_type(declared_type, self.state).emit(
            value,
            self.state,
            block,
        );
        let pointer = Pointer::stack(AstType::new(declared_type), self.state, block);
        pointer.store(AstValue::new(cast), self.state, block);
        self.environment
            .define_variable(identifier.name(), pointer.into_mlir(), declared_type);
    }
}
