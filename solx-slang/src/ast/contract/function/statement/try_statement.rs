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

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TryStatement;

use solx_mlir::ffi;

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
            match call_emitter.emit_external_call_try(call, block) {
                Ok(triple) => triple,
                // Not an external call we can lower with try semantics — run
                // the success body only.
                Err(_) => return self.emit_try_success_only(try_statement, block),
            }
        };

        let (then_block, else_block) = self.state.builder.emit_sol_if(status, &current_block);
        let then_region = ffi::block_parent_region(&then_block);
        let else_region = ffi::block_parent_region(&else_block);
        let saved_region = self.region_pointer;

        // Success region: bind declared returns from the call results, run body.
        self.set_region(&then_region);
        let then_entry = then_block;
        if let Some(parameters) = try_statement.returns() {
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
                    .emit(*result, &self.state.builder, &then_entry);
                let pointer = self
                    .state
                    .builder
                    .emit_sol_alloca(parameter_type, &then_entry);
                self.state.builder.emit_sol_store(cast, pointer, &then_entry);
                self.environment
                    .define_variable(identifier.name(), pointer, parameter_type);
            }
        }
        let then_end = self.emit_block(try_statement.body().statements(), then_entry)?;
        if let Some(end) = then_end {
            self.state.builder.emit_sol_yield(&end);
        }

        // Failure region: run the first catch clause's body (if any).
        self.set_region(&else_region);
        let else_entry = else_block;
        let mut else_end = Some(else_entry);
        if let Some(catch_clause) = try_statement.catch_clauses().iter().next() {
            // Bind the catch error parameter from the failure-path revert data.
            // The low-level form `catch (bytes memory s)` binds `s` to the raw
            // returndata (materialised by `sol.get_returndata`). Typed forms
            // (`catch Error(string r)`, `catch Panic(uint c)`) additionally need the
            // returndata ABI-decoded past the 4-byte selector and are not yet bound.
            if let Some(error) = catch_clause.error()
                && let Some(parameter) = error.parameters().iter().next()
                && let Some(name_identifier) = parameter.name()
            {
                if error.name().is_some() {
                    anyhow::bail!(
                        "typed try/catch with a bound parameter (catch Error/Panic) \
                         is not yet supported"
                    );
                }
                let builder = &self.state.builder;
                let parameter_type = parameter
                    .get_type()
                    .map(|slang_type| {
                        TypeConversion::resolve_slang_type(
                            &slang_type,
                            Some(solx_utils::DataLocation::Memory),
                            builder,
                        )
                    })
                    .unwrap_or(builder.types.sol_string_memory);
                let returndata = else_entry
                    .append_operation(
                        solx_mlir::ods::sol::GetReturnDataOperation::builder(
                            builder.context,
                            builder.unknown_location,
                        )
                        .addr(builder.types.string(solx_utils::DataLocation::Memory))
                        .build()
                        .into(),
                    )
                    .result(0)
                    .expect("sol.get_returndata produces one result")
                    .into();
                let pointer = builder.emit_sol_alloca(parameter_type, &else_entry);
                builder.emit_sol_store(returndata, pointer, &else_entry);
                self.environment
                    .define_variable(name_identifier.name(), pointer, parameter_type);
            }
            else_end = self.emit_block(catch_clause.body().statements(), else_entry)?;
        }
        if let Some(end) = else_end {
            self.state.builder.emit_sol_yield(&end);
        }

        self.region_pointer = saved_region;
        Ok(Some(current_block))
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
