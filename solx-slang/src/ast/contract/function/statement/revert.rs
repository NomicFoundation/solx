//! Revert statement lowering.

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::RevertStatement;

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
        let mut evaluated = match revert.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => {
                self.emit_revert_argument_values(positional.iter(), block)?
            }
            ArgumentsDeclaration::NamedArguments(named) => {
                let ordered = Self::order_named_arguments(&named, &parameters)?;
                self.emit_revert_argument_values(ordered, block)?
            }
        };
        for (value, parameter) in evaluated.values.iter_mut().zip(parameters.iter()) {
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                &self.state.builder,
            );
            *value = TypeConversion::from_target_type(parameter_type, &self.state.builder).emit(
                *value,
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
        let signature: String = match message_argument {
            None => String::new(),
            Some(Expression::StringExpression(string_expression)) => {
                let message = String::from_utf8(string_expression.value())
                    .expect("revert message is valid UTF-8");
                if message.is_empty() {
                    unimplemented!(
                        "revert(\"\") would emit ambiguous bytecode under the current Sol dialect; use revert() for no-data revert"
                    );
                }
                message
            }
            Some(_) => unimplemented!("revert message must be a string literal"),
        };
        self.state
            .builder
            .emit_sol_revert(&signature, &[], false, &block);
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
