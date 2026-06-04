//!
//! Inline array literal expression lowering: `[a, b, c]`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an inline array literal `[a, b, c]` to `sol.array_lit`, coercing
    /// each element to the literal's element type.
    pub fn emit_array_literal(
        &self,
        array_expression: &ArrayExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_type = array_expression
            .get_type()
            .expect("the binder types every array literal");
        let element_slang_type = match &result_type {
            SlangType::FixedSizeArray(array_type) => array_type.element_type(),
            SlangType::Array(array_type) => array_type.element_type(),
            other => unreachable!(
                "an array literal has an array type; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let builder = &self.state.builder;
        let element_type = TypeConversion::resolve_slang_type(&element_slang_type, None, builder);
        let array_type = TypeConversion::resolve_slang_type(&result_type, None, builder);

        let mut elements = Vec::new();
        let mut block = block;
        for item in array_expression.items().iter() {
            let (value, next) = self.emit_value(&item, block)?;
            let value =
                TypeConversion::from_target_type(element_type, builder).emit(value, builder, &next);
            elements.push(value);
            block = next;
        }
        let value = builder.emit_sol_array_lit(&elements, array_type, &block);
        Ok((value, block))
    }
}
