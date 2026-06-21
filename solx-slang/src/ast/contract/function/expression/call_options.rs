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
    // A call-options expression in value position (decorated but not immediately
    // called) contributes only its options' side effects; its value is that of
    // the wrapped operand.
    let mut current_block = block;
    for option in node.options().iter() {
        let BlockAnd { value: _value, block: next } = option.value().emit(context, current_block);
        current_block = next;
    }
    node.operand().emit(context, current_block)
});

/// A `f{value: v, gas: g, salt: s}` call-options layer, viewed for the call
/// modifiers it captures from the option list. `CallOptionsExpression` is a
/// foreign Slang node, so the capture lives on this local lens over it.
pub struct CallOptions<'node>(pub &'node CallOptionsExpression);

impl CallOptions<'_> {
    /// Evaluates the option list in source order (each value emitted for its side
    /// effects) and returns the captured `value` (as `msg.value`, coerced to
    /// `ui256`) and `salt` (the CREATE2 salt for `new`, cast from `bytes32`). The
    /// option KIND comes from slang's typed `BuiltIn::CallOption*` classification,
    /// never from comparing the option name as text. The `{gas: …}` option is not
    /// yet threaded into the call op and is deferred loudly.
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
            // Emit each option toward the type that option expects, so a literal
            // folds correctly: `value`/`gas` are `ui256`, the CREATE2 `salt` is
            // `bytes32` (a hex/string literal `salt: hex"00"` must fold to a
            // fixedbytes constant, NOT a memory string the salt bridge can't take).
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
                    // The gas limit is evaluated for its side effects but not
                    // threaded into the call: the call forwards all remaining gas
                    // (the `sol.ext_icall` default). A `{gas: …}` that must actually
                    // cap the forwarded gas is not yet modelled.
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
