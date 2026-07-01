//! Revert statement lowering.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::Parameters;
use slang_solidity_v2::ast::RevertStatement;

use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::RevertOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_statement::EmitStatement;

/// Identifier the parser uses to recognize the Solidity `revert` built-in.
pub const IDENTIFIER: &str = "revert";

/// Revert arguments evaluated in ABI order.
struct EvaluatedRevertArguments<'context, 'block> {
    /// Evaluated argument values.
    values: Vec<Value<'context, 'block>>,
    /// Current block after evaluating all arguments.
    block: BlockRef<'context, 'block>,
}

statement_emit!(RevertStatement; |node, context, block| {
    let error = match node.error().resolve_to_definition() {
        None => {
            context.append_sol_revert("", &[], false, &block);
            return Some(block);
        }
        Some(Definition::Error(error)) => error,
        Some(_) => unreachable!("revert target resolves to an error definition"),
    };
    let signature = error
        .compute_canonical_signature()
        .expect("canonical signature is computable for a custom error");
    let parameters = error.parameters();
    let mut evaluated = match node.arguments() {
        ArgumentsDeclaration::PositionalArguments(positional) => {
            context.emit_revert_argument_values(positional.iter(), block)
        }
        ArgumentsDeclaration::NamedArguments(named) => {
            let ordered = StatementContext::order_named_revert_arguments(&named, &parameters);
            context.emit_revert_argument_values(ordered, block)
        }
    };
    for (value, parameter) in evaluated.values.iter_mut().zip(parameters.iter()) {
        let parameter_type = TypeConversion::resolve_slang_type(
            &parameter
                .get_type()
                .expect("parameter type resolved by semantic analysis"),
            None,
            context.state,
        );
        *value = TypeConversion::from_target_type(parameter_type, context.state).emit(
            *value,
            context.state,
            &evaluated.block,
        );
    }
    context.append_sol_revert(&signature, &evaluated.values, true, &evaluated.block);
    Some(evaluated.block)
});

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits a `sol.revert` for the call form `revert()`, `revert({})`,
    /// `revert("message")`, or `revert(expression)`.
    ///
    /// A no-data revert covers `revert()` and the degenerate empty
    /// named-argument list `revert({})`. A non-empty string literal bakes its
    /// message into the op. Any other message is evaluated at runtime and
    /// ABI-encoded under the `Error(string)` selector via the `call` form.
    ///
    /// `sol.revert` is not a terminator at the dialect level, so codegen continues in the same
    /// block; the function epilogue (or an enclosing region's yield) supplies the structural
    /// terminator.
    pub(super) fn emit_revert_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        let message_argument = match &call.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional_arguments) => {
                let mut arguments = positional_arguments.iter();
                let message_argument = arguments.next();
                assert!(
                    arguments.next().is_none(),
                    "revert accepts at most one argument"
                );
                message_argument
            }
            ArgumentsDeclaration::NamedArguments(named_arguments) => {
                assert!(
                    named_arguments.iter().next().is_none(),
                    "revert accepts only an empty named-argument list"
                );
                None
            }
        };
        let block = match message_argument {
            None => {
                self.append_sol_revert("", &[], false, &block);
                block
            }
            Some(Expression::StringExpression(string_expression))
                if !string_expression.value().is_empty() =>
            {
                let message = String::from_utf8(string_expression.value())
                    .expect("revert message is valid UTF-8");
                self.append_sol_revert(&message, &[], false, &block);
                block
            }
            Some(expression) => {
                let expression_context = self.expression_context();
                let BlockAnd { value, block } = expression.emit(&expression_context, block);
                let string_memory_type =
                    AstType::string(self.state.mlir_context, solx_utils::DataLocation::Memory)
                        .into_mlir();
                let message_value = TypeConversion::from_target_type(string_memory_type, self.state)
                    .emit(value, self.state, &block);
                self.append_sol_revert("Error(string)", &[message_value], true, &block);
                block
            }
        };
        Some(block)
    }

    /// Appends a `sol.revert` carrying `signature` and the evaluated `args`; `is_custom_error` marks
    /// a custom-error revert with the `call` unit attribute.
    fn append_sol_revert(
        &self,
        signature: &str,
        args: &[Value<'context, 'block>],
        is_custom_error: bool,
        block: &BlockRef<'context, 'block>,
    ) {
        let mut operation_builder =
            RevertOperation::builder(self.state.mlir_context, self.state.location())
                .signature(StringAttribute::new(self.state.mlir_context, signature))
                .args(args);
        if is_custom_error {
            operation_builder = operation_builder.call(Attribute::unit(self.state.mlir_context));
        }
        block.append_operation(operation_builder.build().into());
    }

    /// Orders named revert arguments by the custom error's parameter declaration order.
    fn order_named_revert_arguments(
        named_arguments: &NamedArguments,
        error_parameters: &Parameters,
    ) -> Vec<Expression> {
        let mut arguments = HashMap::new();
        for argument in named_arguments.iter() {
            match arguments.entry(argument.name().name()) {
                Entry::Vacant(entry) => {
                    entry.insert(argument.value());
                }
                Entry::Occupied(entry) => {
                    unreachable!("slang rejects a duplicate named revert argument `{}`", entry.key());
                }
            }
        }

        let mut ordered_arguments = Vec::new();
        for parameter in error_parameters.iter() {
            let parameter_name = parameter
                .name()
                .expect("a named-argument custom error has named parameters")
                .name();
            let argument = arguments
                .remove(&parameter_name)
                .expect("slang matches every named revert argument to a parameter");
            ordered_arguments.push(argument);
        }

        ordered_arguments
    }

    /// Evaluates revert argument expressions left-to-right, threading the
    /// current MLIR block through each evaluation.
    fn emit_revert_argument_values<Arguments>(
        &self,
        arguments: Arguments,
        block: BlockRef<'context, 'block>,
    ) -> EvaluatedRevertArguments<'context, 'block>
    where
        Arguments: IntoIterator<Item = Expression>,
    {
        let expression_context = self.expression_context();
        let mut values = Vec::new();
        let mut current_block = block;
        for argument in arguments {
            let BlockAnd {
                value,
                block: next_block,
            } = argument.emit(&expression_context, current_block);
            values.push(value);
            current_block = next_block;
        }

        EvaluatedRevertArguments {
            values,
            block: current_block,
        }
    }
}
