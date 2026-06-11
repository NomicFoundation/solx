//!
//! Conditional, tuple, and array-literal expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Expression;
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
        // A ternary whose branches are bare function names yields an *internal*
        // function pointer, but slang types it from the function's visibility —
        // a `Public` function as its return type — not the pointer type. The
        // branches emit `func_ref` values, so recover the internal-pointer type
        // from a branch when present; otherwise the conditional's own type is
        // authoritative, falling back to a branch's type when the binder leaves
        // the conditional untyped, rather than silently defaulting to `ui256`
        // (which masked the mismatch and `sol.cast`-ed a `func_ref` to integer).
        let result_type = self
            .bare_function_ref_type(&conditional.true_expression())
            .or_else(|| self.bare_function_ref_type(&conditional.false_expression()))
            .or_else(|| self.resolve_slang_type(conditional.get_type()))
            .or_else(|| self.resolve_slang_type(conditional.true_expression().get_type()))
            .or_else(|| self.resolve_slang_type(conditional.false_expression().get_type()))
            .expect("a conditional resolves its type from itself or one of its branches");
        let condition = conditional.operand();
        let (condition_value, block) = self.emit_value(&condition, block)?;
        let condition_boolean = self.emit_is_nonzero(condition_value, &block);

        let result_slot = self.state.builder.emit_sol_alloca(result_type, &block);
        let (then_block, else_block) = self.state.builder.emit_sol_if(condition_boolean, &block);

        let true_expression = conditional.true_expression();
        let (then_value, then_end) =
            self.emit_value_for_target(&true_expression, result_type, then_block)?;
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
        let (else_value, else_end) =
            self.emit_value_for_target(&false_expression, result_type, else_block)?;
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

    /// Emits a tuple-valued conditional `cond ? (a, …) : (b, …)`, producing one
    /// value per tuple element. Mirrors [`Self::emit_conditional`] but allocates
    /// a slot per element; both branches store into the shared slots and the
    /// loads after the `sol.if` yield the result tuple. Reached only from a
    /// multi-value return position, so the binder guarantees both branches are
    /// equal-length, non-empty tuples.
    pub fn emit_conditional_tuple_values(
        &self,
        conditional: &ConditionalExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // Only literal-tuple branches (`cond ? (1, 2) : (3, 4)`) lower here; a
        // branch that is itself a multi-value call (`cond ? f(a) : g(a)`) needs
        // per-branch multi-result expansion and is deferred (as dev-experimental
        // also defers it).
        let (Expression::TupleExpression(true_tuple), Expression::TupleExpression(false_tuple)) = (
            conditional.true_expression(),
            conditional.false_expression(),
        ) else {
            unimplemented!(
                "multi-value return of a conditional whose branches are not literal tuples"
            );
        };
        let true_items: Vec<Expression> = true_tuple
            .items()
            .iter()
            .filter_map(|item| item.expression())
            .collect();
        let false_items: Vec<Expression> = false_tuple
            .items()
            .iter()
            .filter_map(|item| item.expression())
            .collect();
        assert!(
            !true_items.is_empty() && true_items.len() == false_items.len(),
            "a multi-value conditional's branches are equal-length, non-empty tuples"
        );
        let builder = &self.state.builder;
        let result_types: Vec<_> = true_items
            .iter()
            .map(|item| {
                self.resolve_slang_type(item.get_type())
                    .expect("slang types every conditional-branch tuple element")
            })
            .collect();

        let (condition_value, block) = self.emit_value(&conditional.operand(), block)?;
        let condition_boolean = self.emit_is_nonzero(condition_value, &block);
        let slots: Vec<Value<'context, 'block>> = result_types
            .iter()
            .map(|&result_type| builder.emit_sol_alloca(result_type, &block))
            .collect();
        let (then_block, else_block) = builder.emit_sol_if(condition_boolean, &block);

        for (branch_block, items) in [(then_block, &true_items), (else_block, &false_items)] {
            let mut current = branch_block;
            for (index, item) in items.iter().enumerate() {
                let (value, next) = self.emit_value(item, current)?;
                current = next;
                let cast = TypeConversion::from_target_type(result_types[index], builder)
                    .emit(value, builder, &current);
                builder.emit_sol_store(cast, slots[index], &current);
            }
            builder.emit_sol_yield(&current);
        }

        let mut values = Vec::with_capacity(slots.len());
        for (index, &slot) in slots.iter().enumerate() {
            values.push(builder.emit_sol_load(slot, result_types[index], &block)?);
        }
        Ok((values, block))
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
        // An array literal is always a memory aggregate, so its reference
        // elements live in memory — a `calldata`/`storage` reference element
        // (e.g. a calldata slice `[b[i:j]]`) is copied in. Resolve the element
        // and result types in their memory representation so the per-element
        // coercion below is a `data_loc_cast` into memory (matching solc),
        // rather than leaving a calldata element inside a memory `sol.array_lit`
        // that the backend cannot lower.
        let declared_element_type =
            TypeConversion::resolve_slang_type_in_memory(&element_slang_type, builder);
        // Emit the element values before fixing the element type: for a
        // function-pointer array literal the emitted values are authoritative.
        // A bare function name lowers to an internal `func_ref`, but slang types
        // the literal from the function's `Public` visibility, which resolves to
        // `ext_func_ref` — so the declared element type disagrees with the value.
        // Adopt the value's function-ref type when it does, and rebuild the
        // array type to match (otherwise the per-element coercion casts a
        // function ref through the integer-only `sol.cast`, which the verifier
        // rejects).
        let mut element_values = Vec::new();
        let mut current = block;
        for item in array_expression.items().iter() {
            let (value, next) = self.emit_value(&item, current)?;
            element_values.push(value);
            current = next;
        }
        let element_type = match element_values.first() {
            Some(&first)
                if solx_mlir::TypeFactory::is_sol_function_ref(first.r#type())
                    && first.r#type() != declared_element_type =>
            {
                first.r#type()
            }
            _ => declared_element_type,
        };
        let array_type = match &result_slang_type {
            SlangType::FixedSizeArray(fixed_array_type)
                if element_type != declared_element_type =>
            {
                builder.types.array(
                    solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    solx_utils::DataLocation::Memory,
                )
            }
            _ => TypeConversion::resolve_slang_type_in_memory(&result_slang_type, builder),
        };
        let element_values: Vec<_> = element_values
            .into_iter()
            .map(|value| {
                TypeConversion::from_target_type(element_type, builder)
                    .emit(value, builder, &current)
            })
            .collect();
        let value = builder.emit_sol_array_lit(&element_values, array_type, &current);
        Ok((value, current))
    }
}
