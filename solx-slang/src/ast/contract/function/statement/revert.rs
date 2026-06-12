//! Revert statement lowering.

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::RevertStatement;

use crate::ast::arguments_declaration_ext::ArgumentsDeclarationExt;
use crate::ast::contract::function::statement::StatementEmitter;
use crate::ast::type_conversion::TypeConversion;

/// Revert arguments evaluated in ABI order.
struct EvaluatedRevertArguments<'context, 'block> {
    /// Evaluated argument values.
    values: Vec<Value<'context, 'block>>,
    /// Current block after evaluating all arguments.
    block: BlockRef<'context, 'block>,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a `sol.revert` for a `revert ErrorName(args);` statement.
    ///
    /// # Errors
    ///
    /// Returns an error if the error path resolves to a non-Error definition,
    /// the canonical signature cannot be computed, named arguments cannot be
    /// matched to error parameters, or any argument expression cannot be
    /// lowered.
    ///
    /// # Returns
    ///
    /// Returns `None`: `sol.revert` is a block terminator, so the current block
    /// is complete and codegen does not continue in it (no epilogue or enclosing
    /// yield is appended after the revert).
    pub fn emit_revert(
        &self,
        revert: &RevertStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let error = match revert.error().resolve_to_definition() {
            None => {
                self.state.builder.emit_sol_revert("", &[], false, &block);
                return Ok(None);
            }
            Some(Definition::Error(error)) => error,
            Some(_) => unreachable!("slang resolves a revert target to an error definition"),
        };
        let signature = error
            .compute_canonical_signature()
            .expect("slang computes a canonical signature for an error");
        let parameters = error.parameters();
        let parameter_ids = parameters
            .iter()
            .map(|parameter| parameter.node_id())
            .collect::<Vec<_>>();
        let ordered = revert.arguments().ordered_by(&parameter_ids);
        let mut evaluated = self.emit_revert_argument_values(ordered, block)?;
        for (value, parameter) in evaluated.values.iter_mut().zip(parameters.iter()) {
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                &self.state.builder,
            );
            *value = TypeConversion::coerce(
                *value,
                parameter_type,
                &self.state.builder,
                &evaluated.block,
            );
        }
        self.state
            .builder
            .emit_sol_revert(&signature, &evaluated.values, true, &evaluated.block);
        Ok(None)
    }

    /// Emits a `sol.revert` for the call form `revert()` or `revert("message")`.
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are not positional, more than one
    /// argument is supplied, the message argument is not a string literal, or
    /// the message is empty (which would emit ambiguous bytecode under the
    /// current Sol dialect; `revert()` is the no-data form).
    ///
    /// # Returns
    ///
    /// Returns `None`: `sol.revert` is a block terminator, so the current block
    /// is complete and codegen does not continue in it.
    pub fn emit_revert_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("only positional arguments supported");
        };
        let mut arguments = positional_arguments.iter();
        let message_argument = arguments.next();
        assert!(
            arguments.next().is_none(),
            "revert accepts at most one argument"
        );
        match message_argument {
            None => {
                self.state.builder.emit_sol_revert("", &[], false, &block);
            }
            // A non-empty string literal bakes the message into the op as the
            // `Error(string)` payload (no runtime encoding).
            Some(Expression::StringExpression(string_expression))
                if !string_expression.value().is_empty() =>
            {
                let message = String::from_utf8(string_expression.value())
                    .expect("revert message is valid UTF-8");
                self.state
                    .builder
                    .emit_sol_revert(&message, &[], false, &block);
            }
            // A non-literal message (`revert(expr)`) or an empty literal
            // (`revert("")`, which is `Error("")` — selector + an empty string,
            // NOT a no-data revert) is evaluated at runtime and ABI-encoded under
            // the `Error(string)` selector, exactly like `require(cond, expr)`.
            Some(expression) => {
                let emitter = self.expression_emitter();
                let (message_value, block) = emitter.emit_value(&expression, block)?;
                let builder = &self.state.builder;
                let string_memory_type = builder.types.string(solx_utils::DataLocation::Memory);
                let message_value =
                    TypeConversion::coerce(message_value, string_memory_type, builder, &block);
                builder.emit_sol_revert("Error(string)", &[message_value], true, &block);
            }
        }
        Ok(None)
    }

    /// Evaluates revert argument expressions left-to-right, threading the
    /// current MLIR block through each evaluation.
    ///
    /// # Errors
    ///
    /// Returns an error if any argument expression cannot be lowered.
    fn emit_revert_argument_values<Arguments>(
        &self,
        arguments: Arguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<EvaluatedRevertArguments<'context, 'block>>
    where
        Arguments: IntoIterator<Item = Expression>,
    {
        let emitter = self.expression_emitter();
        let mut values = Vec::new();
        let mut current_block = block;
        for argument in arguments {
            let (value, next_block) = emitter.emit_value(&argument, current_block)?;
            values.push(value);
            current_block = next_block;
        }

        Ok(EvaluatedRevertArguments {
            values,
            block: current_block,
        })
    }
}
