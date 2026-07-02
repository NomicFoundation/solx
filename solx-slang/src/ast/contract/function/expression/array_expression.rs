//!
//! Array-literal expression emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::LocationPolicy;
use solx_mlir::Type as AstType;
use solx_mlir::ods::sol::ArrayLitOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(ArrayExpression; |node, context, block| {
    let result_slang_type = node.get_type().expect("slang validated");
    let element_slang_type = match &result_slang_type {
        SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
        SlangType::Array(array_type) => array_type.element_type(),
        _ => unreachable!(
            "slang types an array literal as Array or FixedSizeArray: {:?}",
            std::mem::discriminant(&result_slang_type)
        ),
    };
    let state = context.state;
    let declared_element_type =
        AstType::resolve(&element_slang_type, LocationPolicy::ForceMemory, state);
    // Emit element values before fixing the element type: for a function-pointer array literal the
    // emitted values are authoritative, since slang types the literal from visibility and can
    // disagree, so adopt the value's function-ref type when it differs and rebuild the array type.
    let mut element_values = Vec::new();
    let mut current = block;
    for item in node.items().iter() {
        let BlockAnd { value, block: next } = item.emit(context, current);
        element_values.push(value);
        current = next;
    }
    let element_type = match element_values.first() {
        Some(&first)
            if first.r#type().is_function_ref()
                && first.r#type().into_mlir() != declared_element_type =>
        {
            first.r#type().into_mlir()
        }
        _ => declared_element_type,
    };
    let array_type = match &result_slang_type {
        SlangType::FixedSizeArray(fixed_array_type) if element_type != declared_element_type => {
            AstType::array(
                state.mlir_context,
                solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                element_type,
                solx_utils::DataLocation::Memory,
            )
            .into_mlir()
        }
        _ => AstType::resolve(&result_slang_type, LocationPolicy::ForceMemory, state),
    };
    let element_values: Vec<_> = element_values
        .into_iter()
        .map(|value| {
            value
                .cast(AstType::new(element_type), state, &current)
                .into_mlir()
        })
        .collect();
    let value: Value<'context, 'block> = mlir_op!(
        state,
        &current,
        ArrayLitOperation.ins(&element_values).addr(array_type)
    );
    BlockAnd { block: current, value: value.into() }
});
