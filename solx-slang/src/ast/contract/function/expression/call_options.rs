//!
//! Call-options expression emission in value position: `f{value: v}` decorated
//! but not immediately called.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Expression;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::Type as AstType;
use crate::ast::contract::function::expression::ExpressionContext;

expression_emit!(CallOptionsExpression; |node, context, block| {
    // In value position (decorated but not called), contributes only its options' side effects;
    // its value is the wrapped operand's.
    let mut current_block = block;
    for option in node.options().iter() {
        let BlockAnd { value: _value, block: next } = option.value().emit(context, current_block);
        current_block = next;
    }
    node.operand().emit(context, current_block)
});

/// A local lens over a foreign `CallOptionsExpression`, capturing the call modifiers (`value` / `salt`).
pub struct CallOptions<'node>(pub &'node CallOptionsExpression);

impl CallOptions<'_> {
    /// Evaluates the option list in source order and returns the captured `value` (as `msg.value`,
    /// coerced to `ui256`) and `salt` (CREATE2 salt, from `bytes32`). `{gas: …}` is not yet threaded.
    pub fn capture<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Option<Value<'context, 'block>>,
        Option<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let mut value = None;
        let mut salt = None;
        let mut current_block = block;
        for option in self.0.options().iter() {
            // Emit each option toward its expected type so a literal folds correctly (the CREATE2
            // `salt` is `bytes32`, so `salt: hex"00"` folds to a fixedbytes constant, not a memory string).
            match option.name().resolve_to_built_in() {
                Some(BuiltIn::CallOptionValue) => {
                    let BlockAnd {
                        value: option_value,
                        block: next_block,
                    } = option.value().emit(context, current_block);
                    current_block = next_block;
                    let builder = &context.state.builder;
                    value = Some(
                        option_value
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                                builder,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                Some(BuiltIn::CallOptionSalt) => {
                    let bytes32 =
                        AstType::fixed_bytes(context.state.builder.context, 32).into_mlir();
                    let salt_expression = option.value();
                    let BlockAnd {
                        value: salt_bytes,
                        block: next_block,
                    } = if let Expression::StringExpression(string_literal) = &salt_expression {
                        string_literal.emit_as(bytes32, context, current_block)
                    } else {
                        salt_expression.emit(context, current_block)
                    };
                    current_block = next_block;
                    let builder = &context.state.builder;
                    salt = Some(
                        salt_bytes
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                                builder,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                Some(BuiltIn::CallOptionGas) => {
                    // The gas limit is evaluated for its side effects but not threaded into the call
                    // (which forwards all remaining gas); a `{gas: …}` that caps gas is not yet modelled.
                    let BlockAnd {
                        value: _gas,
                        block: next_block,
                    } = option.value().emit(context, current_block);
                    current_block = next_block;
                }
                _ => unreachable!("a call option resolves to a value, gas, or salt built-in"),
            }
        }
        (value, salt, current_block)
    }
}
