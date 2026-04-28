//! Revert statement lowering.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionCallExpression;
use slang_solidity::backend::ir::ast::NamedArguments;
use slang_solidity::backend::ir::ast::Parameters;
use slang_solidity::backend::ir::ast::RevertStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::statement::StatementEmitter;

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
    /// # Returns None
    ///
    /// Always returns `None` because `sol.revert` terminates control flow.
    pub fn emit_revert(
        &self,
        revert: &RevertStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let error = match revert.error().resolve_to_definition() {
            None => {
                self.state.builder.emit_sol_revert("", &[], false, &block);
                // TODO(sol-dialect): remove once `sol.revert` is marked `IsTerminator`.
                block.append_operation(melior::dialect::llvm::unreachable(
                    self.state.builder.unknown_location,
                ));
                return Ok(None);
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
        let evaluated = match revert.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                self.emit_revert_argument_values(positional.iter(), block)?
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                let ordered = Self::order_named_revert_arguments(&named, &parameters)?;
                self.emit_revert_argument_values(ordered, block)?
            }
        };
        self.state
            .builder
            .emit_sol_revert(&signature, &evaluated.values, true, &evaluated.block);
        // TODO(sol-dialect): remove once `sol.revert` is marked `IsTerminator`.
        evaluated
            .block
            .append_operation(melior::dialect::llvm::unreachable(
                self.state.builder.unknown_location,
            ));
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
    /// # Returns None
    ///
    /// Always returns `None` because `sol.revert` terminates control flow.
    pub fn emit_revert_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
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
        self.state
            .builder
            .emit_sol_revert(&signature, &[], false, &block);
        // TODO(sol-dialect): remove once `sol.revert` is marked `IsTerminator`.
        block.append_operation(melior::dialect::llvm::unreachable(
            self.state.builder.unknown_location,
        ));
        Ok(None)
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
        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
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
