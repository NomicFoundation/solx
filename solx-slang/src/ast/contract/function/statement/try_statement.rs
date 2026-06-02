//! Try-catch statement lowering.
//!
//! `try recv.f(args) returns (...) { body } catch { handler }` lowers to a
//! `sol.ext_icall` with `try_call` set (which yields a success flag instead
//! of reverting), then a `sol.if` on that flag: the success region binds the
//! decoded returns and runs `body`; the failure region runs the catch
//! `handler`. A catch error parameter (`catch (bytes memory reason)`,
//! `catch Error(string r)`, `catch Panic(uint c)`) is not yet bound — such a
//! handler fails with a clear diagnostic rather than panicking on the
//! unregistered-local lookup; a parameter-less `catch { … }` handler runs.

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::r#type::IntegerType;

use slang_solidity_v2::ast::CatchClause;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TryStatement;

use solx_mlir::CmpPredicate;
use solx_mlir::ods::sol::DecodeOperation;
use solx_mlir::ods::sol::GetReturnDataOperation;
use solx_mlir::ods::yul::ReturnDataCopyOperation as YulReturnDataCopyOp;
use solx_mlir::ods::yul::ReturnDataSizeOperation as YulReturnDataSizeOp;
use solx_mlir::ods::yul::RevertOperation as YulRevertOp;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Binds a `try`'s declared `returns (...)` from the decoded call `results`
    /// — each cast to its declared type — into the current (success-region)
    /// scope. Unnamed returns and missing results are skipped.
    fn bind_try_returns(
        &mut self,
        try_statement: &TryStatement,
        results: &[melior::ir::Value<'context, 'block>],
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
            let cast = TypeConversion::from_target_type(parameter_type, &self.state.builder)
                .emit(*result, &self.state.builder, then_entry);
            let pointer = self
                .state
                .builder
                .emit_sol_alloca(parameter_type, then_entry);
            self.state.builder.emit_sol_store(cast, pointer, then_entry);
            self.environment
                .define_variable(identifier.name(), pointer, parameter_type);
        }
    }

    /// Lowers a `try` statement.
    pub fn emit_try(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();

        // Only external calls can be `try`-ed. Recognise `recv.f(args)` and
        // `recv.f{value: v}(args)`; anything else falls back to running just
        // the success body (no real catch path).
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

        // Emit the external call with try semantics → (status, results).
        let (status, results, current_block) = {
            let emitter = ExpressionEmitter::new(
                self.state,
                self.environment,
                self.storage_layout,
                self.checked,
            );
            let call_emitter = CallEmitter::new(&emitter);
            // `None` = not a try-lowerable call shape (a normal outcome, not an
            // error) — run the success body only.
            match call_emitter.emit_external_call_try(call, block)? {
                Some(triple) => triple,
                None => return self.emit_try_success_only(try_statement, block),
            }
        };

        let (then_block, else_block) = self.state.builder.emit_sol_if(status, &current_block);
        let then_region = then_block.parent_region().expect("block belongs to a region");
        let else_region = else_block.parent_region().expect("block belongs to a region");
        let saved_region = self.region_pointer;

        // Success region: bind declared returns from the call results, run body.
        self.set_region(&then_region);
        let then_entry = then_block;
        self.bind_try_returns(try_statement, &results, &then_entry);
        let then_end = self.emit_block(try_statement.body().statements(), then_entry)?;
        if let Some(end) = then_end {
            self.state.builder.emit_sol_yield(&end);
        }

        // Failure region: dispatch the revert data to the matching catch clause
        // by its 4-byte error selector. `catch Error(string)` matches
        // 0x08c379a0 and `catch Panic(uint)` matches 0x4e487b71; the low-level
        // `catch (bytes)` / parameter-less `catch {}` clause handles anything
        // else (or runs unconditionally when there are no typed clauses).
        self.set_region(&else_region);

        let clauses: Vec<CatchClause> = try_statement.catch_clauses().iter().collect();
        let mut typed: Vec<(i64, CatchClause)> = Vec::new();
        let mut fallback: Option<CatchClause> = None;
        for clause in clauses {
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
            // selector = uint32(abi.decode(returndata, (bytes4))) — the error
            // selector is the high 4 bytes of the first returndata word.
            let selector = {
                let builder = &self.state.builder;
                let bytes = self.emit_returndata_decode(
                    0,
                    &[builder.types.fixed_bytes(4)],
                    &else_cursor,
                )[0];
                builder.emit_sol_cast(bytes, ui32, &else_cursor)
            };
            for (selector_constant, clause) in &typed {
                let (then_block, next_else, then_region, next_else_region) = {
                    let builder = &self.state.builder;
                    let expected = builder.emit_sol_constant(*selector_constant, ui32, &else_cursor);
                    let matches =
                        builder.emit_sol_cmp(selector, expected, CmpPredicate::Eq, &else_cursor);
                    let (then_block, next_else) = builder.emit_sol_if(matches, &else_cursor);
                    let then_region = then_block.parent_region().expect("block belongs to a region");
                    let next_else_region = next_else.parent_region().expect("block belongs to a region");
                    // The `sol.if` is the last op of `else_cursor`'s block; it
                    // still needs a terminator (the structured-if op does not
                    // terminate the enclosing block).
                    builder.emit_sol_yield(&else_cursor);
                    (then_block, next_else, then_region, next_else_region)
                };
                self.set_region(&then_region);
                self.emit_typed_catch_clause(clause, then_block)?;
                else_cursor = next_else;
                self.set_region(&next_else_region);
            }
        }

        self.emit_fallback_catch_clause(fallback.as_ref(), else_cursor)?;

        self.region_pointer = saved_region;
        Ok(Some(current_block))
    }

    /// Materialises `returndata[start..]` and ABI-decodes it into `result_types`.
    fn emit_returndata_decode(
        &self,
        start: i64,
        result_types: &[Type<'context>],
        block: &BlockRef<'context, 'block>,
    ) -> Vec<melior::ir::Value<'context, 'block>> {
        let builder = &self.state.builder;
        let start_value = builder.emit_sol_constant(start, builder.types.ui256, block);
        let returndata = block
            .append_operation(
                GetReturnDataOperation::builder(builder.context, builder.unknown_location)
                    .start(start_value)
                    .addr(builder.types.string(solx_utils::DataLocation::Memory))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.get_returndata produces one result")
            .into();
        let operation = block.append_operation(
            DecodeOperation::builder(builder.context, builder.unknown_location)
                .addr(returndata)
                .outs(result_types)
                .build()
                .into(),
        );
        (0..result_types.len())
            .map(|index| {
                operation
                    .result(index)
                    .expect("sol.decode yields one result per requested type")
                    .into()
            })
            .collect()
    }

    /// Emits a typed catch clause (`catch Error(string r)` / `catch Panic(uint c)`):
    /// binds the parameter by ABI-decoding the revert data past the 4-byte
    /// selector, then runs the body and yields.
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
        let end = self.emit_block(clause.body().statements(), block)?;
        if let Some(end) = end {
            self.state.builder.emit_sol_yield(&end);
        }
        Ok(())
    }

    /// Emits the fallback catch clause (low-level `catch (bytes memory s)` or
    /// parameter-less `catch {}`), or an empty yield when there is none. The
    /// low-level form binds the whole revert data.
    fn emit_fallback_catch_clause(
        &mut self,
        clause: Option<&CatchClause>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let Some(clause) = clause else {
            // No catch clause matched the revert selector (only typed clauses,
            // none of which applied): propagate by re-reverting the exact
            // revert data. `yul.revert` is not a terminator, so a dead
            // `sol.yield` follows to terminate the structured region.
            let builder = &self.state.builder;
            let i256 = Type::from(IntegerType::new(builder.context, 256));
            let size = block
                .append_operation(
                    YulReturnDataSizeOp::builder(builder.context, builder.unknown_location)
                        .out(i256)
                        .build()
                        .into(),
                )
                .result(0)
                .expect("yul.returndatasize produces one result")
                .into();
            let zero_unsigned = builder.emit_sol_constant(0, builder.types.ui256, &block);
            let zero = builder.emit_sol_cast(zero_unsigned, i256, &block);
            block.append_operation(
                YulReturnDataCopyOp::builder(builder.context, builder.unknown_location)
                    .dst(zero)
                    .src(zero)
                    .size(size)
                    .build()
                    .into(),
            );
            block.append_operation(
                YulRevertOp::builder(builder.context, builder.unknown_location)
                    .addr(zero)
                    .size(size)
                    .build()
                    .into(),
            );
            builder.emit_sol_yield(&block);
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
            let start_value =
                self.state
                    .builder
                    .emit_sol_constant(0, self.state.builder.types.ui256, &block);
            let returndata = block
                .append_operation(
                    GetReturnDataOperation::builder(
                        self.state.builder.context,
                        self.state.builder.unknown_location,
                    )
                    .start(start_value)
                    .addr(self.state.builder.types.string(solx_utils::DataLocation::Memory))
                    .build()
                    .into(),
                )
                .result(0)
                .expect("sol.get_returndata produces one result")
                .into();
            let builder = &self.state.builder;
            let pointer = builder.emit_sol_alloca(parameter_type, &block);
            builder.emit_sol_store(returndata, pointer, &block);
            self.environment
                .define_variable(name_identifier.name(), pointer, parameter_type);
        }
        let end = self.emit_block(clause.body().statements(), block)?;
        if let Some(end) = end {
            self.state.builder.emit_sol_yield(&end);
        }
        Ok(())
    }

    /// Fallback used when the try expression is not an external call we can
    /// lower with try semantics: emit the call and the success body, ignoring
    /// the catch clause.
    fn emit_try_success_only(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();
        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (value, current_block) = emitter.emit(&expression, block)?;

        if let Some(parameters) = try_statement.returns()
            && let Some(value) = value
            && let Some(parameter) = parameters.iter().next()
            && let Some(name_identifier) = parameter.name()
        {
            let name = name_identifier.name();
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
            self.state.builder.emit_sol_store(cast, pointer, &current_block);
            self.environment.define_variable(name, pointer, parameter_type);
        }

        self.emit_block(try_statement.body().statements(), current_block)
    }
}
