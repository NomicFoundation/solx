//!
//! Conditional, tuple, and array-literal expression emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::ArrayLitOperation;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Materialize;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope>
    for ConditionalExpression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>);

    /// Emits `cond ? a : b`, yielding one value per result: a scalar ternary
    /// yields a single value, a tuple-valued conditional (`cond ? (x, y) : (z, w)`,
    /// reached only from a multi-value position) one per tuple element. Both
    /// branches store into shared result slots and the loads after the `sol.if`
    /// yield the result(s). In value position a scalar conditional is dispatched
    /// through [`Expression`]'s emit, which takes the sole value.
    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output {
        let true_expression = self.true_expression();
        let false_expression = self.false_expression();

        // A tuple-valued conditional yields one value per tuple element. A branch
        // is either a literal tuple (`(a, b, c)`) whose element types come from its
        // items, or a multi-value expression — a tuple-returning call (`this.f(a)`)
        // or a nested conditional — whose element types come from the conditional's
        // own tuple type. Each branch is expanded to one value per result slot;
        // both branches store into the shared slots before the loads.
        if let Some(SlangType::Tuple(tuple_type)) = self.get_type() {
            let result_types: Vec<Type<'context>> = match (&true_expression, &false_expression) {
                (
                    Expression::TupleExpression(true_tuple),
                    Expression::TupleExpression(false_tuple),
                ) => {
                    let true_items: Vec<Expression> = true_tuple
                        .items()
                        .iter()
                        .filter_map(|item| item.expression())
                        .collect();
                    let false_count = false_tuple
                        .items()
                        .iter()
                        .filter_map(|item| item.expression())
                        .count();
                    assert!(
                        !true_items.is_empty() && true_items.len() == false_count,
                        "a multi-value conditional's branches are equal-length, non-empty tuples"
                    );
                    true_items
                        .iter()
                        .map(|item| {
                            AstType::resolve_optional(item.get_type(), &context.state.builder)
                                .expect("slang types every conditional-branch tuple element")
                        })
                        .collect()
                }
                _ => tuple_type
                    .types()
                    .iter()
                    .map(|element_type| {
                        AstType::resolve_optional(
                            Some(element_type.clone()),
                            &context.state.builder,
                        )
                        .expect("slang types every conditional result element")
                    })
                    .collect(),
            };

            let builder = &context.state.builder;
            let BlockAnd {
                value: condition_value,
                block,
            } = self.operand().emit(context, block);
            let condition_boolean = condition_value.is_nonzero(builder, &block).into_mlir();
            let slots: Vec<Pointer<'context, 'block>> = result_types
                .iter()
                .map(|&result_type| Pointer::stack_slot(AstType::new(result_type), builder, &block))
                .collect();
            let (then_block, else_block) = sol_region_op!(builder, &block, IfOperation.cond(condition_boolean); then_region, else_region);

            for (branch_block, branch_expression) in [
                (then_block, &true_expression),
                (else_block, &false_expression),
            ] {
                // Expand the branch to one value per result slot: a literal tuple
                // yields each item's value; a tuple-returning call its full result
                // list (like `return f();`); a nested conditional its own values.
                let (values, current) = match branch_expression {
                    Expression::TupleExpression(tuple) => {
                        let mut values = Vec::new();
                        let mut current = branch_block;
                        for item in tuple.items().iter() {
                            let inner = item.expression().expect(
                                "a multi-value conditional tuple element has an inner expression",
                            );
                            let BlockAnd { value, block: next } = inner.emit(context, current);
                            values.push(value.into_mlir());
                            current = next;
                        }
                        (values, current)
                    }
                    Expression::FunctionCallExpression(call) => call.emit(context, branch_block),
                    Expression::ConditionalExpression(nested) => nested.emit(context, branch_block),
                    other => unimplemented!(
                        "multi-value conditional branch of this expression kind is not supported: {:?}",
                        std::mem::discriminant(other)
                    ),
                };
                assert!(
                    values.len() == slots.len(),
                    "a conditional branch yields one value per result slot"
                );
                for (index, value) in values.into_iter().enumerate() {
                    let cast = AstValue::from(value).cast(
                        AstType::new(result_types[index]),
                        builder,
                        &current,
                    );
                    slots[index].store(cast, builder, &current);
                }
                sol_op_void!(builder, &current, YieldOperation.ins(&[]));
            }

            let mut values = Vec::with_capacity(slots.len());
            for (index, &slot) in slots.iter().enumerate() {
                values.push(
                    slot.load(AstType::new(result_types[index]), builder, &block)
                        .into_mlir(),
                );
            }
            return (values, block);
        }

        // A scalar ternary yields a single value. A branch of bare function names
        // yields an *internal* function pointer, but slang types it from the
        // function's visibility — a `Public` function as its return type — not the
        // pointer type. The branches emit `func_ref` values, so recover the
        // internal-pointer type from a branch when present; otherwise the
        // conditional's own type is authoritative, falling back to a branch's type
        // when the binder leaves the conditional untyped, rather than silently
        // defaulting to `ui256` (which masked the mismatch and `sol.cast`-ed a
        // `func_ref` to integer).
        let result_type = context
            .bare_function_ref_type(&true_expression)
            .or_else(|| context.bare_function_ref_type(&false_expression))
            .or_else(|| AstType::resolve_optional(self.get_type(), &context.state.builder))
            .or_else(|| {
                AstType::resolve_optional(true_expression.get_type(), &context.state.builder)
            })
            .or_else(|| {
                AstType::resolve_optional(false_expression.get_type(), &context.state.builder)
            })
            .expect("a conditional resolves its type from itself or one of its branches");
        let BlockAnd {
            value: condition_value,
            block,
        } = self.operand().emit(context, block);
        let condition_boolean = condition_value
            .is_nonzero(&context.state.builder, &block)
            .into_mlir();

        let result_slot =
            Pointer::stack_slot(AstType::new(result_type), &context.state.builder, &block);
        let (then_block, else_block) = sol_region_op!(
            &context.state.builder, &block,
            IfOperation.cond(condition_boolean); then_region, else_region
        );

        let BlockAnd {
            value: then_value,
            block: then_end,
        } = if let Expression::StringExpression(string_literal) = &true_expression {
            string_literal.materialize(result_type, context, then_block)
        } else {
            true_expression.emit(context, then_block)
        };
        let then_cast =
            then_value.cast(AstType::new(result_type), &context.state.builder, &then_end);
        result_slot.store(then_cast, &context.state.builder, &then_end);
        sol_op_void!(&context.state.builder, &then_end, YieldOperation.ins(&[]));

        let BlockAnd {
            value: else_value,
            block: else_end,
        } = if let Expression::StringExpression(string_literal) = &false_expression {
            string_literal.materialize(result_type, context, else_block)
        } else {
            false_expression.emit(context, else_block)
        };
        let else_cast =
            else_value.cast(AstType::new(result_type), &context.state.builder, &else_end);
        result_slot.store(else_cast, &context.state.builder, &else_end);
        sol_op_void!(&context.state.builder, &else_end, YieldOperation.ins(&[]));

        let result = result_slot.load(AstType::new(result_type), &context.state.builder, &block);
        (vec![result.into_mlir()], block)
    }
}

expression_emit!(TupleExpression; |node, context, block| {
    let items = node.items();
    // TODO: support multi-value tuples (e.g. tuple deconstruction)
    assert!(items.len() == 1, "multi-value tuples not yet supported");
    let item = items.iter().next().expect("length checked to be 1 above");
    let inner = item
        .expression()
        .expect("a single-element tuple has an inner expression");
    inner.emit(context, block)
});

expression_emit!(ArrayExpression; |node, context, block| {
    let result_slang_type = node.get_type().expect("slang types every array literal");
    let element_slang_type = match &result_slang_type {
        SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
        SlangType::Array(array_type) => array_type.element_type(),
        _ => unreachable!(
            "slang types an array literal as Array or FixedSizeArray: {:?}",
            std::mem::discriminant(&result_slang_type)
        ),
    };
    let builder = &context.state.builder;
    // An array literal is always a memory aggregate, so its reference
    // elements live in memory — a `calldata`/`storage` reference element
    // (e.g. a calldata slice `[b[i:j]]`) is copied in. Resolve the element
    // and result types in their memory representation so the per-element
    // coercion below is a `data_loc_cast` into memory (matching solc),
    // rather than leaving a calldata element inside a memory `sol.array_lit`
    // that the backend cannot lower.
    let declared_element_type =
        AstType::resolve(&element_slang_type, LocationPolicy::ForceMemory, builder);
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
                builder.context,
                solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                element_type,
                solx_utils::DataLocation::Memory,
            )
            .into_mlir()
        }
        _ => AstType::resolve(&result_slang_type, LocationPolicy::ForceMemory, builder),
    };
    let element_values: Vec<_> = element_values
        .into_iter()
        .map(|value| {
            value
                .cast(AstType::new(element_type), builder, &current)
                .into_mlir()
        })
        .collect();
    let value: Value<'context, 'block> = sol_op!(
        builder,
        &current,
        ArrayLitOperation.ins(&element_values).addr(array_type)
    );
    BlockAnd { block: current, value: value.into() }
});
