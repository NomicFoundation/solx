//!
//! Call-options expression emission in value position: `f{value: v}` decorated
//! but not immediately called.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Expression;

use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(CallOptionsExpression; |node, context, block| {
    let mut current_block = block;
    for option in node.options().iter() {
        let BlockAnd { value: _value, block: next } = option.value().emit(context, current_block);
        current_block = next;
    }
    node.operand().emit(context, current_block)
});

/// A local lens over a foreign `CallOptionsExpression`, capturing the `value`, `salt`, and `gas`
/// call modifiers.
pub struct CallOptions<'node>(pub &'node CallOptionsExpression);

impl CallOptions<'_> {
    /// Evaluates the option list in source order and returns `(value, salt, gas, block)` in that
    /// tuple order. A dropped `value`/`salt`/`gas` is `None`; the caller then sends zero value/salt
    /// and forwards all remaining gas.
    pub fn capture<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Option<Value<'context, 'block>>,
        Option<Value<'context, 'block>>,
        Option<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let mut value = None;
        let mut salt = None;
        let mut gas = None;
        let mut current_block = block;
        for option in self.0.options().iter() {
            match option.name().resolve_to_built_in() {
                Some(BuiltIn::CallOptionValue) => {
                    let BlockAnd {
                        value: option_value,
                        block: next_block,
                    } = option.value().emit(context, current_block);
                    current_block = next_block;
                    let state = context.state;
                    value = Some(
                        AstValue::new(option_value)
                            .cast(
                                AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                                state,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                Some(BuiltIn::CallOptionSalt) => {
                    let bytes32 = AstType::fixed_bytes(context.state.mlir_context, 32).into_mlir();
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
                    let state = context.state;
                    salt = Some(
                        AstValue::new(salt_bytes)
                            .bytes_cast(
                                AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                                state,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                Some(BuiltIn::CallOptionGas) => {
                    let BlockAnd {
                        value: option_value,
                        block: next_block,
                    } = option.value().emit(context, current_block);
                    current_block = next_block;
                    let state = context.state;
                    gas = Some(
                        AstValue::new(option_value)
                            .cast(
                                AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                                state,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                _ => unreachable!("a call option resolves to a value, gas, or salt built-in"),
            }
        }
        (value, salt, gas, current_block)
    }
}
