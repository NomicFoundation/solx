//! Revert statement emission.

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::RevertStatement;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::arguments_declaration_ext::ArgumentsDeclarationExt;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
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
    /// Returns `Some(block)`: `sol.revert` is not a terminator, so the block
    /// stays live for the caller to terminate (an enclosing yield or the
    /// function epilogue's default return).
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
        let block = match message_argument {
            None => {
                self.state.builder.emit_sol_revert("", &[], false, &block);
                block
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
                block
            }
            // A non-literal message (`revert(expr)`) or an empty literal
            // (`revert("")`, which is `Error("")` — selector + an empty string,
            // NOT a no-data revert) is evaluated at runtime and ABI-encoded under
            // the `Error(string)` selector, exactly like `require(cond, expr)`.
            Some(expression) => {
                let emitter = ExpressionContext::from(self);
                let BlockAnd {
                    value: message_value,
                    block,
                } = expression.emit(&emitter, block)?;
                let builder = &self.state.builder;
                let string_memory_type =
                    crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory)
                        .into_mlir();
                let message_value = message_value
                    .coerce_to(crate::ast::Type::new(string_memory_type), builder, &block)
                    .into_mlir();
                builder.emit_sol_revert("Error(string)", &[message_value], true, &block);
                block
            }
        };
        Ok(Some(block))
    }
}

// `sol.revert` is not a terminator: the block stays live and the caller appends
// its terminator (an enclosing `sol.yield` or the epilogue default return).
statement_emit!(RevertStatement; |node, context, block| {
    let error = match node.error().resolve_to_definition() {
        None => {
            context.state.builder.emit_sol_revert("", &[], false, &block);
            return Ok(Some(block));
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
    let ordered = node.arguments().ordered_by(&parameter_ids);
    let parameter_types: Vec<_> = parameters
        .iter()
        .map(|parameter| {
            parameter
                .get_type()
                .expect("parameter type resolved by semantic analysis")
                .resolve_type(LocationPolicy::Declared(None), &context.state.builder)
        })
        .collect();
    let emitter = ExpressionContext::from(&*context);
    let (values, block) =
        emitter.emit_coerced_argument_expressions(&ordered, &parameter_types, block)?;
    context
        .state
        .builder
        .emit_sol_revert(&signature, &values, true, &block);
    Ok(Some(block))
});
