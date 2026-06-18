//! Revert statement emission.

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::RevertStatement;
use solx_mlir::ods::sol::RevertOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::EmitStatement;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::statement::StatementContext;

impl<'state, 'context, 'block> StatementContext<'state, 'context, 'block> {
    /// Emits a `sol.revert` for the call form `revert()` or `revert("message")`.
    /// # Returns
    ///
    /// Returns `Some(block)`: `sol.revert` is not a terminator, so the block
    /// stays live for the caller to terminate (an enclosing yield or the
    /// function epilogue's default return).
    pub fn emit_revert_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> Option<BlockRef<'context, 'block>> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("only positional arguments supported");
        };
        let message_argument = positional_arguments.iter().next();
        let block = match message_argument {
            None => {
                self.emit_revert("", &[], false, &block);
                block
            }
            // A non-empty string literal bakes the message into the op as the
            // `Error(string)` payload (no runtime encoding).
            Some(Expression::StringExpression(string_expression))
                if !string_expression.value().is_empty() =>
            {
                let message = String::from_utf8(string_expression.value())
                    .expect("revert message is valid UTF-8");
                self.emit_revert(&message, &[], false, &block);
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
                } = expression.emit(&emitter, block);
                let builder = &self.state.builder;
                let string_memory_type =
                    AstType::string(builder.context, solx_utils::DataLocation::Memory).into_mlir();
                let message_value = message_value
                    .cast(AstType::new(string_memory_type), builder, &block)
                    .into_mlir();
                self.emit_revert("Error(string)", &[message_value], true, &block);
                block
            }
        };
        Some(block)
    }

    /// Emits a `sol.revert` carrying an optional payload: `signature` is the
    /// payload string (a custom error's canonical signature, `Error(string)`, a
    /// literal message, or empty for `revert()`), `args` the evaluated operands,
    /// and `is_custom_error` selects the call-encoded form. Not a terminator — the
    /// block stays live for the caller to terminate.
    fn emit_revert(
        &self,
        signature: &str,
        args: &[Value<'context, 'block>],
        is_custom_error: bool,
        block: &BlockRef<'context, 'block>,
    ) {
        let builder = &self.state.builder;
        let mut operation_builder =
            RevertOperation::builder(builder.context, builder.unknown_location)
                .signature(StringAttribute::new(builder.context, signature))
                .args(args);
        if is_custom_error {
            operation_builder = operation_builder.call(Attribute::unit(builder.context));
        }
        block.append_operation(operation_builder.build().into());
    }
}

// `sol.revert` is not a terminator: the block stays live and the caller appends
// its terminator (an enclosing `sol.yield` or the epilogue default return).
statement_emit!(RevertStatement; |node, context, block| {
    let error = match node.error().resolve_to_definition() {
        None => {
            context.emit_revert("", &[], false, &block);
            return Some(block);
        }
        Some(Definition::Error(error)) => error,
        Some(_) => unreachable!("slang resolves a revert target to an error definition"),
    };
    let signature = error
        .compute_canonical_signature()
        .expect("slang validated");
    let parameters = error.parameters();
    let parameter_ids = parameters
        .iter()
        .map(|parameter| parameter.node_id())
        .collect::<Vec<_>>();
    let ordered = node.arguments().ordered_by(&parameter_ids);
    let parameter_types: Vec<_> = parameters
        .iter()
        .map(|parameter| {
            AstType::resolve(
                &parameter
                    .get_type()
                    .expect("slang validated"),
                LocationPolicy::Declared(None),
                &context.state.builder,
            )
        })
        .collect();
    let emitter = ExpressionContext::from(&*context);
    let BlockAnd {
        value: values,
        block,
    } = ordered.emit_as(&parameter_types, &emitter, block);
    context.emit_revert(&signature, &values, true, &block);
    Some(block)
});
