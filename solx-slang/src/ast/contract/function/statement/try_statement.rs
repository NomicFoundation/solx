//!
//! Try-catch statement lowering.
//!
//! `try recv.f(args) returns (...) { body } catch { handler }` lowers to a
//! `sol.ext_icall` with `try_call` set (which yields a success flag instead of
//! reverting), then a `sol.if` on that flag: the success region binds the
//! decoded returns and runs `body`; the failure region dispatches the revert
//! data to the matching catch clause by its 4-byte error selector
//! (`Error(string)` → `0x08c379a0`, `Panic(uint)` → `0x4e487b71`), falling back
//! to the low-level `catch (bytes)` / parameter-less `catch {}` clause, or
//! re-reverting verbatim when only typed clauses are present and none matched.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::TryStatement;

use solx_mlir::CmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `try` statement.
    pub fn emit_try(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();

        // Only an external call can be `try`-ed. Recognise `recv.f(args)` and
        // `recv.f{value: v}(args)`; any other shape just runs the success body.
        let Expression::FunctionCallExpression(call) = &expression else {
            return self.emit_try_success_only(try_statement, block);
        };
        if !matches!(
            call.operand(),
            Expression::MemberAccessExpression(_) | Expression::CallOptionsExpression(_)
        ) {
            return self.emit_try_success_only(try_statement, block);
        }

        // Emit the external call with try semantics → (status, results). `None`
        // means the call is not a try-lowerable shape (a normal outcome, not an
        // error), so fall back to running the success body only.
        let (status, results, current_block) = {
            let emitter = ExpressionEmitter::new(
                self.state,
                self.environment,
                self.storage_layout,
                self.checked,
            );
            match CallEmitter::new(&emitter).emit_external_call_try(call, block)? {
                Some(triple) => triple,
                None => return self.emit_try_success_only(try_statement, block),
            }
        };

        let (then_block, else_block) = self.state.builder.emit_sol_if(status, &current_block);

        // Success region: bind the declared `returns (...)` from the call
        // results, then run the body.
        self.bind_try_returns(try_statement, &results, &then_block);
        self.emit_region_body(try_statement.body().statements(), then_block)?;

        // Failure region: dispatch to the matching catch clause.
        self.emit_catch_clauses(try_statement, else_block)?;

        Ok(Some(current_block))
    }

    /// Binds a `try`'s declared `returns (...)` from the decoded call `results`
    /// — each cast to its declared type — into the success-region scope. Unnamed
    /// returns and missing results are skipped.
    fn bind_try_returns(
        &mut self,
        try_statement: &TryStatement,
        results: &[Value<'context, 'block>],
        then_entry: &BlockRef<'context, 'block>,
    ) {
        let Some(parameters) = try_statement.returns() else {
            return;
        };
        for (parameter, result) in parameters.iter().zip(results.iter()) {
            let Some(identifier) = parameter.name() else {
                continue;
            };
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
                .define_variable(identifier.name(), pointer, parameter_type);
        }
    }

    /// Dispatches the failure region to the matching catch clause by the
    /// external call's 4-byte error selector. Typed `catch Error(string)` /
    /// `catch Panic(uint)` clauses are tested in turn against their selectors;
    /// the low-level `catch (bytes)` / parameter-less `catch {}` clause handles
    /// anything else (or runs unconditionally when there are no typed clauses).
    fn emit_catch_clauses(
        &mut self,
        try_statement: &TryStatement,
        else_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let mut typed: Vec<(i64, CatchClause)> = Vec::new();
        let mut fallback: Option<CatchClause> = None;
        for clause in try_statement.catch_clauses().iter() {
            match clause
                .error()
                .and_then(|error| error.name())
                .map(|name| name.name())
                .as_deref()
            {
                Some("Error") => typed.push((0x08c3_79a0, clause)),
                Some("Panic") => typed.push((0x4e48_7b71, clause)),
                _ => fallback = Some(clause),
            }
        }

        let mut else_cursor = else_block;
        if !typed.is_empty() {
            let ui32 = Type::from(IntegerType::unsigned(self.state.builder.context, 32));
            // selector = uint32(returndata[0..4]) — the error selector occupies
            // the high 4 bytes of the first returndata word.
            let selector = {
                let builder = &self.state.builder;
                let bytes =
                    self.emit_returndata_decode(0, &[builder.types.fixed_bytes(4)], &else_cursor)
                        [0];
                builder.emit_sol_cast(bytes, ui32, &else_cursor)
            };
            for (selector_constant, clause) in &typed {
                let (then_block, next_else) = {
                    let builder = &self.state.builder;
                    let expected =
                        builder.emit_sol_constant(*selector_constant, ui32, &else_cursor);
                    let matches =
                        builder.emit_sol_cmp(selector, expected, CmpPredicate::Eq, &else_cursor);
                    let (then_block, next_else) = builder.emit_sol_if(matches, &else_cursor);
                    // The `sol.if` is the last op of `else_cursor`'s block but is
                    // not itself a terminator, so the block still needs one.
                    builder.emit_sol_yield(&else_cursor);
                    (then_block, next_else)
                };
                self.emit_typed_catch_clause(clause, then_block)?;
                else_cursor = next_else;
            }
        }

        self.emit_fallback_catch_clause(fallback.as_ref(), else_cursor)
    }

    /// Materialises `returndata[start..]` and ABI-decodes it into `result_types`.
    fn emit_returndata_decode(
        &self,
        start: i64,
        result_types: &[Type<'context>],
        block: &BlockRef<'context, 'block>,
    ) -> Vec<Value<'context, 'block>> {
        let builder = &self.state.builder;
        let start_value = builder.emit_sol_constant(start, builder.types.ui256, block);
        let returndata = builder.emit_sol_get_returndata(start_value, block);
        builder.emit_sol_decode(returndata, result_types, block)
    }

    /// Emits a typed catch clause (`catch Error(string r)` / `catch Panic(uint c)`):
    /// binds its parameter by ABI-decoding the revert data past the 4-byte
    /// selector, then runs the body.
    fn emit_typed_catch_clause(
        &mut self,
        clause: &CatchClause,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        if let Some(error) = clause.error()
            && let Some(parameter) = error.parameters().iter().next()
            && let Some(name_identifier) = parameter.name()
        {
            let parameter_type = parameter
                .get_type()
                .map(|slang_type| {
                    TypeConversion::resolve_slang_type(
                        &slang_type,
                        Some(solx_utils::DataLocation::Memory),
                        &self.state.builder,
                    )
                })
                .unwrap_or(self.state.builder.types.ui256);
            let decoded = self.emit_returndata_decode(4, &[parameter_type], &block)[0];
            let builder = &self.state.builder;
            let pointer = builder.emit_sol_alloca(parameter_type, &block);
            builder.emit_sol_store(decoded, pointer, &block);
            self.environment
                .define_variable(name_identifier.name(), pointer, parameter_type);
        }
        self.emit_region_body(clause.body().statements(), block)
    }

    /// Emits the fallback catch clause (low-level `catch (bytes memory s)` or
    /// parameter-less `catch {}`), or re-reverts with the exact revert data when
    /// there is none (only typed clauses applied, none matched). The low-level
    /// form binds the whole revert data.
    fn emit_fallback_catch_clause(
        &mut self,
        clause: Option<&CatchClause>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let Some(clause) = clause else {
            // No catch clause matched the revert selector: propagate by
            // re-reverting the exact revert data. The `yul.revert` is not a
            // terminator, so a dead `sol.yield` follows to terminate the region.
            self.state.builder.emit_revert_returndata(&block);
            self.state.builder.emit_sol_yield(&block);
            return Ok(());
        };
        if let Some(error) = clause.error()
            && let Some(parameter) = error.parameters().iter().next()
            && let Some(name_identifier) = parameter.name()
        {
            let parameter_type = parameter
                .get_type()
                .map(|slang_type| {
                    TypeConversion::resolve_slang_type(
                        &slang_type,
                        Some(solx_utils::DataLocation::Memory),
                        &self.state.builder,
                    )
                })
                .unwrap_or(self.state.builder.types.sol_string_memory);
            let start =
                self.state
                    .builder
                    .emit_sol_constant(0, self.state.builder.types.ui256, &block);
            let returndata = self.state.builder.emit_sol_get_returndata(start, &block);
            let builder = &self.state.builder;
            let pointer = builder.emit_sol_alloca(parameter_type, &block);
            builder.emit_sol_store(returndata, pointer, &block);
            self.environment
                .define_variable(name_identifier.name(), pointer, parameter_type);
        }
        self.emit_region_body(clause.body().statements(), block)
    }

    /// Emits a try/catch region body (`statements`) into `entry` and terminates
    /// its region — yielding the fall-through block, or appending a dead
    /// yielding block when the body itself terminated (via `return`/`break`/
    /// `continue`), matching the always-yield region shape of `if`/loop lowering.
    fn emit_region_body(
        &mut self,
        statements: Statements,
        entry: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let region = entry
            .parent_region()
            .expect("a try/catch region body block belongs to a region");
        match self.emit_block(statements, entry)? {
            Some(end) => self.state.builder.emit_sol_yield(&end),
            None => self.emit_dead_yield(&region),
        }
        Ok(())
    }

    /// Fallback for a `try` whose expression is not an external call we can lower
    /// with try semantics: emit the call and the success body, ignoring the catch
    /// clauses (there is no real failure path to dispatch).
    fn emit_try_success_only(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();
        let (value, current_block) = {
            let emitter = ExpressionEmitter::new(
                self.state,
                self.environment,
                self.storage_layout,
                self.checked,
            );
            emitter.emit(&expression, block)?
        };

        if let Some(parameters) = try_statement.returns()
            && let Some(value) = value
            && let Some(parameter) = parameters.iter().next()
            && let Some(name_identifier) = parameter.name()
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
                .define_variable(name_identifier.name(), pointer, parameter_type);
        }

        self.emit_block(try_statement.body().statements(), current_block)
    }
}
