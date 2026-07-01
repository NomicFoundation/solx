//!
//! Array-literal expression emission.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(ArrayExpression; |node, context, block| {
    let result_slang_type = node
        .get_type()
        .expect("slang types every array literal");
    let element_slang_type = match &result_slang_type {
        SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
        SlangType::Array(array_type) => array_type.element_type(),
        _ => unreachable!(
            "array literal has unexpected result type: {:?}",
            std::mem::discriminant(&result_slang_type)
        ),
    };
    let array_type =
        TypeConversion::resolve_slang_type(&result_slang_type, None, context.state);
    let element_type =
        TypeConversion::resolve_slang_type(&element_slang_type, None, context.state);
    let mut element_values = Vec::new();
    let mut current = block;
    for item in node.items().iter() {
        let BlockAnd { value, block: next } = item.emit(context, current);
        let cast_value = TypeConversion::from_target_type(element_type, context.state)
            .emit(value, context.state, &next);
        element_values.push(cast_value);
        current = next;
    }
    let value = AstValue::array_literal(
        &element_values,
        AstType::new(array_type),
        context.state,
        &current,
    )
    .into_mlir();
    BlockAnd {
        block: current,
        value,
    }
});
