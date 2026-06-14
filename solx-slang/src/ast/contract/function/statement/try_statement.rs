//!
//! `try` statement lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::TryStatement;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::TryFallbackKind;
use solx_mlir::ods::sol::StoreOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::try_external_call::TryExternalCall;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits a `try` statement as a `sol.try`: an external call with try
    /// semantics produces the success `status`, and the op carries four regions
    /// — success, panic, error, fallback. The success region binds the declared
    /// `returns (...)` and runs the body; each present catch clause populates its
    /// region, receiving the lowering-decoded panic code / error reason / raw
    /// returndata as a block argument. Absent clauses leave empty regions, and
    /// the op's lowering owns the selector dispatch, payload decode, and raw
    /// re-revert when no clause matches.
    pub fn emit_try(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();

        // Only a lowerable external call (`recv.f(args)` / `recv.f{value: v}(args)`)
        // carries a real catch path; any other `try` expression runs only the
        // success body.
        let Some(try_call) = TryExternalCall::classify(&expression) else {
            return self.emit_try_success_only(try_statement, block);
        };
        let (status, results, current_block) =
            try_call.emit(&ExpressionContext::from(&*self), block)?;

        // Classify the catch clauses into the `sol.try` regions, all structurally
        // (Rule-7): a parameter-less `catch {}` has no error group; a low-level
        // `catch (bytes r)` has an error group with no name; a typed
        // `catch Error(...)` / `catch Panic(...)` names its error, and Error vs
        // Panic is told apart by its bound parameter's type (`Error(string)` vs
        // `Panic(uint256)`), never by the identifier text.
        let mut panic_clause: Option<CatchClause> = None;
        let mut error_clause: Option<CatchClause> = None;
        let mut fallback_clause: Option<CatchClause> = None;
        let mut fallback_kind = TryFallbackKind::None;
        for clause in try_statement.catch_clauses().iter() {
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
                        _ => unreachable!(
                            "a typed catch clause binds Error(string) or Panic(uint256)"
                        ),
                    }
                }
            }
        }

        let saved_region = self.region_pointer;
        let (success_block, panic_block, error_block, fallback_block) =
            self.state.builder.emit_sol_try(
                status,
                panic_clause.is_some(),
                error_clause.is_some(),
                fallback_kind,
                &current_block,
            );

        // Success region: bind the declared returns from the call results, then
        // run the body.
        let success_region = success_block
            .parent_region()
            .expect("block belongs to a region");
        self.set_region(&success_region);
        self.bind_try_returns(try_statement, &results, &success_block);
        let success_end = self.emit_block(try_statement.body().statements(), success_block)?;
        if let Some(end) = success_end {
            sol_op_void!(&self.state.builder, &end, YieldOperation.ins(&[]));
        }

        // Typed `catch Panic(uint)` / `catch Error(string)`: the decoded value
        // arrives as the region's block argument.
        if let Some(panic_block) = panic_block {
            let clause = panic_clause.expect("a panic region implies a panic clause");
            self.emit_typed_catch_clause(&clause, panic_block)?;
        }
        if let Some(error_block) = error_block {
            let clause = error_clause.expect("an error region implies an error clause");
            self.emit_typed_catch_clause(&clause, error_block)?;
        }
        // The single low-level `catch (bytes r)` / parameter-less `catch {}`.
        if let Some(fallback_block) = fallback_block {
            let clause = fallback_clause.expect("a fallback region implies a fallback clause");
            self.emit_fallback_catch_clause(&clause, fallback_kind, fallback_block)?;
        }

        self.region_pointer = saved_region;
        Ok(Some(current_block))
    }

    /// Binds the declared `returns (...)` of a `try` from the decoded results —
    /// each cast to its declared type — into the current (success-region) scope.
    /// Unnamed returns and missing results are skipped.
    pub fn bind_try_returns(
        &mut self,
        try_statement: &TryStatement,
        results: &[Value<'context, 'block>],
        then_entry: &BlockRef<'context, 'block>,
    ) {
        let Some(parameters) = try_statement.returns() else {
            return;
        };
        for (parameter, result) in parameters.iter().zip(results.iter()) {
            if parameter.name().is_none() {
                continue;
            }
            self.bind_catch_parameter(&parameter, *result, then_entry);
        }
    }

    /// Casts a lowering-decoded catch value to the bound parameter's declared
    /// type, stores it into a fresh stack slot, and defines the parameter in
    /// scope by its node id — the shared binding for the success `returns (...)`,
    /// the typed `Error`/`Panic` reason/code, and the low-level `bytes` data.
    fn bind_catch_parameter(
        &mut self,
        parameter: &Parameter,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        let parameter_type = parameter
            .get_type()
            .map(|slang_type| {
                slang_type.resolve_type(LocationPolicy::Declared(None), &self.state.builder)
            })
            .unwrap_or_else(|| {
                crate::ast::Type::unsigned(self.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir()
            });
        let cast = crate::ast::Value::from(value)
            .coerce_to(
                crate::ast::Type::new(parameter_type),
                &self.state.builder,
                block,
            )
            .into_mlir();
        let pointer = crate::ast::Pointer::stack_slot(
            crate::ast::Type::new(parameter_type),
            &self.state.builder,
            block,
        )
        .into_mlir();
        sol_op_void!(
            &self.state.builder,
            block,
            StoreOperation.val(cast).addr(pointer)
        );
        self.environment
            .define_variable(parameter.node_id(), pointer);
    }

    /// Emits a typed `catch Error(string memory r)` / `catch Panic(uint c)`
    /// clause. The lowering decodes the reason / code and delivers it as the
    /// region's block argument, which is bound to the clause parameter before
    /// the body runs.
    pub fn emit_typed_catch_clause(
        &mut self,
        clause: &CatchClause,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let region = block.parent_region().expect("block belongs to a region");
        self.set_region(&region);
        let error = clause
            .error()
            .expect("a typed catch clause has an error group");
        let parameter = error
            .parameters()
            .iter()
            .next()
            .expect("a typed catch clause declares one parameter");
        let decoded: Value<'context, 'block> = block.argument(0)?.into();
        self.bind_catch_parameter(&parameter, decoded, &block);
        let end = self.emit_block(clause.body().statements(), block)?;
        if let Some(end) = end {
            sol_op_void!(&self.state.builder, &end, YieldOperation.ins(&[]));
        }
        Ok(())
    }

    /// Emits the fallback clause. A parameter-less `catch { ... }` runs its body
    /// directly; a low-level `catch (bytes memory r)` binds `r` to the raw
    /// returndata delivered as the region's block argument. The no-clause-matched
    /// re-revert is synthesised by the `sol.try` lowering when the fallback
    /// region is empty, so it is never emitted here.
    pub fn emit_fallback_catch_clause(
        &mut self,
        clause: &CatchClause,
        kind: TryFallbackKind,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let region = block.parent_region().expect("block belongs to a region");
        self.set_region(&region);
        if let TryFallbackKind::Bytes = kind {
            let error = clause
                .error()
                .expect("a low-level catch clause has a parameter group");
            let parameter = error
                .parameters()
                .iter()
                .next()
                .expect("a low-level catch clause binds one bytes parameter");
            let data: Value<'context, 'block> = block.argument(0)?.into();
            self.bind_catch_parameter(&parameter, data, &block);
        }
        let end = self.emit_block(clause.body().statements(), block)?;
        if let Some(end) = end {
            sol_op_void!(&self.state.builder, &end, YieldOperation.ins(&[]));
        }
        Ok(())
    }

    /// The non-try-lowerable fallback: emit the call, bind the first declared
    /// return, then the body (the catch clauses are unreachable here).
    pub fn emit_try_success_only(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();
        let BlockAnd {
            value,
            block: current_block,
        } = {
            let emitter = ExpressionContext::from(&*self);
            expression.emit(&emitter, block)?
        };

        if let Some(parameters) = try_statement.returns()
            && let Some(parameter) = parameters.iter().next()
            && parameter.name().is_some()
        {
            let parameter_type = parameter
                .get_type()
                .map(|slang_type| {
                    slang_type.resolve_type(LocationPolicy::Declared(None), &self.state.builder)
                })
                .unwrap_or_else(|| {
                    crate::ast::Type::unsigned(
                        self.state.builder.context,
                        solx_utils::BIT_LENGTH_FIELD,
                    )
                    .into_mlir()
                });
            let cast = value
                .coerce_to(
                    crate::ast::Type::new(parameter_type),
                    &self.state.builder,
                    &current_block,
                )
                .into_mlir();
            let pointer = crate::ast::Pointer::stack_slot(
                crate::ast::Type::new(parameter_type),
                &self.state.builder,
                &current_block,
            )
            .into_mlir();
            sol_op_void!(
                &self.state.builder,
                &current_block,
                StoreOperation.val(cast).addr(pointer)
            );
            self.environment
                .define_variable(parameter.node_id(), pointer);
        }

        self.emit_block(try_statement.body().statements(), current_block)
    }
}

statement_emit!(TryStatement; |node, context, block| { context.emit_try(node, block) });
