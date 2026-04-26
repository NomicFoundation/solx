//!
//! Revert statement lowering.
//!

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::NamedArguments;
use slang_solidity::backend::ir::ast::Parameters;
use slang_solidity::backend::ir::ast::RevertStatement;

use super::StatementEmitter;
use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a `sol.revert` with optional custom error signature and arguments.
    pub(super) fn emit_revert(
        &self,
        revert: &RevertStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some((signature, argument_values, current_block)) =
            self.emit_custom_error_revert_data(revert, block)?
        else {
            self.emit_revert_terminator("", &[], false, block);
            return Ok(None);
        };

        self.emit_revert_terminator(&signature, &argument_values, true, current_block);
        Ok(None)
    }

    /// Resolves and evaluates custom error data for a revert statement.
    fn emit_custom_error_revert_data(
        &self,
        revert: &RevertStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<
        Option<(
            String,
            Vec<Value<'context, 'block>>,
            BlockRef<'context, 'block>,
        )>,
    > {
        let Some(Definition::Error(error)) = revert.error().resolve_to_definition() else {
            return Ok(None);
        };

        let signature = error
            .compute_canonical_signature()
            .ok_or_else(|| anyhow::anyhow!("cannot compute canonical signature for error"))?;
        let arguments = revert.arguments();
        let parameters = error.parameters();
        let (argument_values, current_block) =
            self.emit_revert_arguments(&arguments, &parameters, block)?;

        Ok(Some((signature, argument_values, current_block)))
    }

    /// Evaluates positional or named revert arguments in ABI order.
    fn emit_revert_arguments(
        &self,
        arguments: &ArgumentsDeclaration,
        error_parameters: &Parameters,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        match arguments {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                self.emit_revert_argument_values(positional.iter(), block)
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                let arguments = Self::order_named_revert_arguments(named, error_parameters)?;
                self.emit_revert_argument_values(arguments, block)
            }
        }
    }

    /// Orders named revert arguments by the custom error's parameter order.
    fn order_named_revert_arguments(
        named_arguments: &NamedArguments,
        error_parameters: &Parameters,
    ) -> anyhow::Result<Vec<Expression>> {
        let mut arguments = HashMap::new();
        for argument in named_arguments.iter() {
            let name = argument.name().name();
            let previous = arguments.insert(name.clone(), argument.value());
            anyhow::ensure!(
                previous.is_none(),
                "duplicate named revert argument `{name}`"
            );
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
            arguments.keys().cloned().collect::<Vec<_>>().join(", ")
        );

        Ok(ordered_arguments)
    }

    /// Evaluates revert argument expressions from left to right.
    fn emit_revert_argument_values(
        &self,
        arguments: impl IntoIterator<Item = Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let emitter = ExpressionEmitter::new(
            &self.semantic,
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let mut argument_values = Vec::new();
        let mut current_block = block;
        for argument in arguments {
            let (value, next_block) = emitter.emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        Ok((argument_values, current_block))
    }

    /// Emits a revert terminator followed by the temporary unreachable.
    fn emit_revert_terminator(
        &self,
        signature: &str,
        argument_values: &[Value<'context, 'block>],
        is_custom_error: bool,
        block: BlockRef<'context, 'block>,
    ) {
        self.state
            .builder
            .emit_sol_revert(signature, argument_values, is_custom_error, &block);
        // TODO(sol-dialect): remove once sol.revert is marked IsTerminator
        block.append_operation(melior::dialect::llvm::unreachable(
            self.state.builder.unknown_location,
        ));
    }
}
