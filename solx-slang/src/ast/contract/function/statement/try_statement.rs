//!
//! `try` statement lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TryStatement;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::statement::StatementEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a `try` statement: an external call with try semantics, a
    /// `sol.if(status)` on the success flag, the success body (binding the
    /// declared `returns (...)`), and the failure region.
    ///
    /// This fill covers the success path and the parameter-less `catch {}`
    /// clause. The revert-data-binding clauses — low-level `catch (bytes r)`,
    /// typed `catch Error(...)` / `catch Panic(...)` — decode the returndata via
    /// `sol.get_returndata`, which the current dialect pin lacks; those land
    /// with the pin advance in a later fill and are deferred loudly here.
    pub fn emit_try(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();

        // Only external calls (`recv.f(args)` / `recv.f{value: v}(args)`) carry a
        // real catch path; any other `try` expression runs only the success body.
        let is_external_call = matches!(&expression, Expression::FunctionCallExpression(call)
        if matches!(
            call.operand(),
            Expression::MemberAccessExpression(_) | Expression::CallOptionsExpression(_)
        ));
        if !is_external_call {
            return self.emit_try_success_only(try_statement, block);
        }
        let Expression::FunctionCallExpression(call) = &expression else {
            return self.emit_try_success_only(try_statement, block);
        };

        // Typed `catch Error(...)` / `catch Panic(...)` clauses dispatch on the
        // 4-byte error selector (returndata decode + per-selector compare) — a
        // later fill. Detect them structurally — a typed clause names its error,
        // so the discriminator is `error().name().is_some()`, never a comparison
        // of that name as text (Rule-7) — and defer loudly; the success body +
        // parameter-less `catch {}` path is emitted here.
        let has_typed_catch = try_statement
            .catch_clauses()
            .iter()
            .any(|clause| clause.error().and_then(|error| error.name()).is_some());
        if has_typed_catch {
            unimplemented!("typed catch clause (Error / Panic)");
        }

        // Emit the external call with try semantics → (status, results). A
        // non-try-lowerable shape (`None`) falls back to the success body.
        let lowered = {
            let emitter = self.expression_emitter();
            let call_emitter = CallEmitter::new(&emitter);
            call_emitter.emit_external_call_try(call, block)?
        };
        let (status, results, current_block) = match lowered {
            Some(triple) => triple,
            None => return self.emit_try_success_only(try_statement, block),
        };

        let (then_block, else_block) = self.state.builder.emit_sol_if(status, &current_block);
        let then_region = then_block
            .parent_region()
            .expect("block belongs to a region");
        let else_region = else_block
            .parent_region()
            .expect("block belongs to a region");
        let saved_region = self.region_pointer;

        // Success region: bind the declared returns from the call results, then
        // run the body.
        self.set_region(&then_region);
        self.bind_try_returns(try_statement, &results, &then_block);
        let then_end = self.emit_block(try_statement.body().statements(), then_block)?;
        if let Some(end) = then_end {
            self.state.builder.emit_sol_yield(&end);
        }

        // Failure region: the single low-level / parameter-less clause.
        self.set_region(&else_region);
        let fallback = try_statement.catch_clauses().iter().next();
        self.emit_fallback_catch_clause(fallback.as_ref(), else_block)?;

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
            let parameter_type = parameter
                .get_type()
                .map(|slang_type| {
                    TypeConversion::resolve_slang_type(&slang_type, None, &self.state.builder)
                })
                .unwrap_or_else(|| self.state.builder.types.ui256);
            let cast = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                *result,
                &self.state.builder,
                then_entry,
            );
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, then_entry);
            self.state.builder.emit_sol_store(cast, pointer, then_entry);
            self.environment
                .define_variable(parameter.node_id(), pointer, parameter_type);
        }
    }

    /// Decodes the returndata from `start` into `result_types`
    /// (`GetReturnData` + `Decode`) — used by the revert-data-binding catch
    /// clauses. Needs the `sol.get_returndata` op (pin advance), so it lands
    /// with the typed-catch fill.
    pub fn emit_returndata_decode(
        &self,
        start: i64,
        result_types: &[Type<'context>],
        block: &BlockRef<'context, 'block>,
    ) -> Vec<Value<'context, 'block>> {
        let _ = (start, result_types, block);
        unimplemented!("returndata decode")
    }

    /// Emits a typed `catch Error(...)` / `catch Panic(...)` clause (bind the
    /// param past the 4-byte selector, body, yield).
    pub fn emit_typed_catch_clause(
        &mut self,
        clause: &CatchClause,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let _ = (clause, block);
        unimplemented!("typed catch clause")
    }

    /// Emits the fallback clause. A parameter-less `catch {}` simply runs its
    /// body. A low-level `catch (bytes memory r)` binds the whole revert data
    /// via `sol.get_returndata` (pin advance) and is deferred; `None` (no clause
    /// applied) re-reverts the exact data, reachable only once typed clauses
    /// exist, so it too is deferred.
    pub fn emit_fallback_catch_clause(
        &mut self,
        clause: Option<&CatchClause>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let Some(clause) = clause else {
            unimplemented!("re-revert when no catch clause applies");
        };
        if clause.error().is_some() {
            unimplemented!("low-level catch clause with a bound parameter");
        }
        let end = self.emit_block(clause.body().statements(), block)?;
        if let Some(end) = end {
            self.state.builder.emit_sol_yield(&end);
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
        let (value, current_block) = {
            let emitter = self.expression_emitter();
            emitter.emit(&expression, block)?
        };

        if let Some(parameters) = try_statement.returns()
            && let Some(value) = value
            && let Some(parameter) = parameters.iter().next()
            && parameter.name().is_some()
        {
            let parameter_type = parameter
                .get_type()
                .map(|slang_type| {
                    TypeConversion::resolve_slang_type(&slang_type, None, &self.state.builder)
                })
                .unwrap_or_else(|| self.state.builder.types.ui256);
            let cast = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                value,
                &self.state.builder,
                &current_block,
            );
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, &current_block);
            self.state
                .builder
                .emit_sol_store(cast, pointer, &current_block);
            self.environment
                .define_variable(parameter.node_id(), pointer, parameter_type);
        }

        self.emit_block(try_statement.body().statements(), current_block)
    }
}
