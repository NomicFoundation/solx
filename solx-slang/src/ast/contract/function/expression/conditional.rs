//!
//! Conditional, tuple, and array-literal expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::TupleExpression;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a parenthesised single-element tuple expression `(e)` by lowering
    /// its inner expression.
    pub fn emit_tuple(
        &self,
        tuple: &TupleExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let items = tuple.items();
        // TODO: support multi-value tuples (e.g. tuple deconstruction)
        assert!(items.len() == 1, "multi-value tuples not yet supported");
        let item = items.iter().next().expect("length checked to be 1 above");
        let inner = item
            .expression()
            .expect("a single-element tuple has an inner expression");
        self.emit(&inner, block)
    }

    /// Emits a ternary conditional `cond ? a : b` using `sol.if` with an alloca
    /// for the result, matching solc's lowering.
    pub fn emit_conditional(
        &self,
        conditional: &ConditionalExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // The conditional's own type is the common type of its branches; when
        // the binder leaves it untyped (e.g. `cond ? f : g` over public
        // functions, where slang types the whole expression as `None`), fall
        // back to a branch's type rather than silently defaulting to `ui256` —
        // that masked the mismatch and `sol.cast`-ed a `func_ref` to an integer.
        let result_type = self
            .resolve_slang_type(conditional.get_type())
            .or_else(|| self.resolve_slang_type(conditional.true_expression().get_type()))
            .or_else(|| self.resolve_slang_type(conditional.false_expression().get_type()))
            .expect("a conditional resolves its type from itself or one of its branches");
        let condition = conditional.operand();
        let (condition_value, block) = self.emit_value(&condition, block)?;
        let condition_boolean = self.emit_is_nonzero(condition_value, &block);

        let result_slot = self.state.builder.emit_sol_alloca(result_type, &block);
        let (then_block, else_block) = self.state.builder.emit_sol_if(condition_boolean, &block);

        let true_expression = conditional.true_expression();
        let (then_value, then_end) = self.emit_value(&true_expression, then_block)?;
        let then_cast = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            then_value,
            &self.state.builder,
            &then_end,
        );
        self.state
            .builder
            .emit_sol_store(then_cast, result_slot, &then_end);
        self.state.builder.emit_sol_yield(&then_end);

        let false_expression = conditional.false_expression();
        let (else_value, else_end) = self.emit_value(&false_expression, else_block)?;
        let else_cast = TypeConversion::from_target_type(result_type, &self.state.builder).emit(
            else_value,
            &self.state.builder,
            &else_end,
        );
        self.state
            .builder
            .emit_sol_store(else_cast, result_slot, &else_end);
        self.state.builder.emit_sol_yield(&else_end);

        let result = self
            .state
            .builder
            .emit_sol_load(result_slot, result_type, &block)?;

        Ok((result, block))
    }

    /// Emits an array literal `[a, b, c]` as a `sol.array_lit`, casting each
    /// element to the array's element type.
    pub fn emit_array_literal(
        &self,
        array_expression: &ArrayExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_slang_type = array_expression
            .get_type()
            .expect("slang types every array literal");
        let element_slang_type = match &result_slang_type {
            SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
            SlangType::Array(array_type) => array_type.element_type(),
            _ => unreachable!(
                "slang types an array literal as Array or FixedSizeArray: {:?}",
                std::mem::discriminant(&result_slang_type)
            ),
        };
        let builder = &self.state.builder;
        let array_type = TypeConversion::resolve_slang_type(&result_slang_type, None, builder);
        let element_type = TypeConversion::resolve_slang_type(&element_slang_type, None, builder);
        let mut element_values = Vec::new();
        let mut current = block;
        for item in array_expression.items().iter() {
            let (value, next) = self.emit_value(&item, current)?;
            let cast_value =
                TypeConversion::from_target_type(element_type, builder).emit(value, builder, &next);
            element_values.push(cast_value);
            current = next;
        }
        let value = builder.emit_sol_array_lit(&element_values, array_type, &current);
        Ok((value, current))
    }
}
