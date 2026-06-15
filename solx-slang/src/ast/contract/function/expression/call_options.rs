//!
//! Call-options expression emission in value position: `f{value: v}` decorated
//! but not immediately called.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::CallOptionsExpression;

use crate::ast::BlockAnd;
use crate::ast::Emit;
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
