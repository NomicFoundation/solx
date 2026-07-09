//! Revert statement lowering.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameters;
use slang_solidity_v2::ast::RevertStatement;

use solx_mlir::Context;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

/// Identifier the parser uses to recognize the Solidity `revert` built-in.
pub const IDENTIFIER: &str = "revert";

impl<'state, 'context> StatementEmitter<'state, 'context> {
    /// Emits a `sol.revert` for a `revert ErrorName(args);` statement.
    ///
    /// `sol.revert` is not a terminator at the dialect level, so codegen
    /// continues in the same block; the function epilogue (or an enclosing
    /// region's yield) supplies the structural terminator.
    ///
    /// # Errors
    ///
    /// Returns an error if the error path resolves to a non-Error definition,
    /// the canonical signature cannot be computed, named arguments cannot be
    /// matched to error parameters, or any argument expression cannot be
    /// lowered.
    pub fn emit_revert(
        &self,
        revert: &RevertStatement,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let error = match revert.error().resolve_to_definition() {
            None => {
                let block = context.current_block();
                block.revert("", &[], false, context);
                return Ok(());
            }
            Some(Definition::Error(error)) => error,
            Some(_) => anyhow::bail!("revert target does not resolve to an error definition"),
        };
        let signature = error.compute_canonical_signature().ok_or_else(|| {
            anyhow::anyhow!(
                "cannot compute canonical signature for error `{}`",
                error.name().name()
            )
        })?;
        let parameters = error.parameters();
        let mut values = match revert.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                self.emit_revert_argument_values(positional.iter(), context)?
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                let ordered = Self::order_named_revert_arguments(&named, &parameters)?;
                self.emit_revert_argument_values(ordered, context)?
            }
        };
        for (value, parameter) in values.iter_mut().zip(parameters.iter()) {
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                context,
            );
            *value =
                TypeConversion::from_target_type(parameter_type, context).emit(*value, context);
        }
        let block = context.current_block();
        block.revert(&signature, values.as_slice(), true, context);
        Ok(())
    }

    /// Emits a `sol.revert` for the call form `revert()` or `revert("message")`.
    ///
    /// `sol.revert` is not a terminator at the dialect level, so codegen
    /// continues in the same block; the function epilogue (or an enclosing
    /// region's yield) supplies the structural terminator.
    ///
    /// # Errors
    ///
    /// Returns an error if the arguments are not positional, more than one
    /// argument is supplied, the message argument is not a string literal, or
    /// the message is empty (which would emit ambiguous bytecode under the
    /// current Sol dialect; `revert()` is the no-data form).
    pub fn emit_revert_call(
        &self,
        call: &FunctionCallExpression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<()> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };
        let mut arguments = positional_arguments.iter();
        let message_argument = arguments.next();
        anyhow::ensure!(
            arguments.next().is_none(),
            "revert accepts at most one argument"
        );
        let signature: String = match message_argument {
            None => String::new(),
            Some(Expression::StringExpression(string_expression)) => {
                let message = String::from_utf8(string_expression.value())
                    .expect("revert message is valid UTF-8");
                anyhow::ensure!(
                    !message.is_empty(),
                    "revert(\"\") would emit ambiguous bytecode under the current Sol dialect; use revert() for no-data revert"
                );
                message
            }
            Some(_) => anyhow::bail!("revert message must be a string literal"),
        };
        let block = context.current_block();
        block.revert(&signature, &[], false, context);
        Ok(())
    }

    /// Orders named revert arguments by the custom error's parameter declaration order.
    fn order_named_revert_arguments(
        named_arguments: &NamedArguments,
        error_parameters: &Parameters,
    ) -> anyhow::Result<Vec<Expression>> {
        let mut arguments = HashMap::new();
        for argument in named_arguments.iter() {
            match arguments.entry(argument.name().name()) {
                Entry::Vacant(entry) => {
                    entry.insert(argument.value());
                }
                Entry::Occupied(entry) => {
                    anyhow::bail!("duplicate named revert argument `{}`", entry.key());
                }
            }
        }

        let mut ordered_arguments = Vec::new();
        for parameter in error_parameters.iter() {
            let parameter_name = parameter
                .name()
                .ok_or_else(|| {
                    anyhow::anyhow!("cannot match named revert argument to unnamed error parameter")
                })?
                .name();
            let argument = arguments.remove(&parameter_name).ok_or_else(|| {
                anyhow::anyhow!("missing named revert argument `{parameter_name}`")
            })?;
            ordered_arguments.push(argument);
        }

        anyhow::ensure!(
            arguments.is_empty(),
            "unknown named revert argument(s): {}",
            arguments
                .keys()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        );

        Ok(ordered_arguments)
    }

    /// Evaluates revert argument expressions left-to-right at the insertion cursor.
    ///
    /// # Errors
    ///
    /// Returns an error if any argument expression cannot be lowered.
    fn emit_revert_argument_values<Arguments>(
        &self,
        arguments: Arguments,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Vec<Value<'context>>>
    where
        Arguments: IntoIterator<Item = Expression>,
    {
        let emitter = ExpressionEmitter::new(self.environment, self.storage_layout, self.checked);
        let mut values = Vec::new();
        for argument in arguments {
            let value = emitter.emit_value(&argument, context)?;
            values.push(value);
        }
        Ok(values)
    }
}
