//!
//! The `assert` / `require` error-checking built-ins (`sol.assert` /
//! `sol.require`), including `require(cond, "literal")`,
//! `require(cond, runtimeExpr)`, and `require(cond, CustomError(args))`.
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Emits an `assert(condition)` built-in via `sol.assert`.
    pub(crate) fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_assert(condition_boolean, &block);
        Ok(block)
    }

    /// Emits a `require(condition)` or `require(condition, message)` built-in
    /// via `sol.require`.
    ///
    /// Literal string messages lower to `sol.require %cond, "msg" : ()`. A
    /// non-literal expression evaluates at runtime and is ABI-encoded under
    /// the `Error(string)` selector via the `call` form of `sol.require`.
    pub(crate) fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);

        // `require(cond, CustomError(args))` reverts with the custom error's
        // ABI encoding (selector + encoded args) when `cond` is false, exactly
        // like `revert CustomError(args)` but gated on the condition.
        if let Some(Expression::FunctionCallExpression(error_call)) = message
            && let Expression::Identifier(callee) = error_call.operand()
            && let Some(Definition::Error(error_definition)) = callee.resolve_to_definition()
        {
            let signature = error_definition.compute_canonical_signature().ok_or_else(|| {
                anyhow::anyhow!(
                    "cannot compute canonical signature for error `{}`",
                    error_definition.name().name()
                )
            })?;
            let ArgumentsDeclaration::PositionalArguments(error_arguments) =
                error_call.arguments()
            else {
                unimplemented!("named arguments in a `require` custom error are not supported");
            };
            let (mut argument_values, block) =
                self.emit_argument_values(&error_arguments, block)?;
            let parameters = error_definition.parameters();
            let builder = &self.expression_emitter.state.builder;
            for (value, parameter) in argument_values.iter_mut().zip(parameters.iter()) {
                let parameter_type = TypeConversion::resolve_slang_type(
                    &parameter
                        .get_type()
                        .expect("error parameter typed by the binder"),
                    None,
                    builder,
                );
                *value = TypeConversion::from_target_type(parameter_type, builder).emit(
                    *value,
                    builder,
                    &block,
                );
            }
            builder.emit_sol_require(
                condition_boolean,
                Some(&signature),
                &argument_values,
                true,
                &block,
            );
            return Ok(block);
        }

        let builder = &self.expression_emitter.state.builder;
        match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes)
                    .map_err(|_| anyhow::anyhow!("require message contains invalid UTF-8"))?;
                builder.emit_sol_require(condition_boolean, Some(&literal), &[], false, &block);
                Ok(block)
            }
            Some(expression) => {
                let (message_value, block) =
                    self.expression_emitter.emit_value(expression, block)?;
                let string_memory_type = builder.types.string(solx_utils::DataLocation::Memory);
                let message_value = TypeConversion::from_target_type(string_memory_type, builder)
                    .emit(message_value, builder, &block);
                builder.emit_sol_require(
                    condition_boolean,
                    Some("Error(string)"),
                    &[message_value],
                    true,
                    &block,
                );
                Ok(block)
            }
            None => {
                builder.emit_sol_require(condition_boolean, None, &[], false, &block);
                Ok(block)
            }
        }
    }
}
