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
use crate::ast::Toward;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a tuple-valued conditional `cond ? a : b`, producing one value per
    /// tuple element. Mirrors the single-value conditional but allocates a slot
    /// per element; both branches store into the shared slots and the loads
    /// after the `sol.if` yield the result tuple. Reached only from a multi-value
    /// position, so the binder guarantees both branches yield equal-length,
    /// non-empty tuples.
    ///
    /// A branch is either a literal tuple (`(a, b, c)`) whose element types come
    /// from its items, or a multi-value expression — a tuple-returning call
    /// (`this.f(a)`) or a nested conditional — whose element types come from the
    /// conditional's own tuple type. Each branch is expanded to one value per
    /// result slot by [`Self::emit_conditional_branch_values`].
    pub fn emit_conditional_tuple_values(
        &self,
        conditional: &ConditionalExpression,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let true_expression = conditional.true_expression();
        let false_expression = conditional.false_expression();
        let result_types: Vec<Type<'context>> = match (&true_expression, &false_expression) {
            // Both branches are literal tuples: take the element types from
            // the (equal-length) items, exactly as the original emission did.
            (Expression::TupleExpression(true_tuple), Expression::TupleExpression(false_tuple)) => {
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
                        crate::ast::Type::resolve_optional(item.get_type(), &self.state.builder)
                            .expect("slang types every conditional-branch tuple element")
                    })
                    .collect()
            }
            // At least one branch is a multi-value call or nested conditional:
            // there are no literal items, so the element types come from the
            // conditional's own tuple type.
            _ => {
                let SlangType::Tuple(tuple_type) = conditional
                    .get_type()
                    .expect("slang types a multi-value conditional")
                else {
                    unreachable!("a multi-value conditional is typed as a tuple");
                };
                tuple_type
                    .types()
                    .iter()
                    .map(|element_type| {
                        crate::ast::Type::resolve_optional(
                            Some(element_type.clone()),
                            &self.state.builder,
                        )
                        .expect("slang types every conditional result element")
                    })
                    .collect()
            }
        };

        let builder = &self.state.builder;
        let BlockAnd {
            value: condition_value,
            block,
        } = conditional.operand().emit(self, block);
        let condition_boolean = condition_value.is_nonzero(builder, &block).into_mlir();
        let slots: Vec<crate::ast::Pointer<'context, 'block>> = result_types
            .iter()
            .map(|&result_type| {
                crate::ast::Pointer::stack_slot(crate::ast::Type::new(result_type), builder, &block)
            })
            .collect();
        let (then_block, else_block) = sol_region_op!(builder, &block, IfOperation.cond(condition_boolean); then_region, else_region);

        for (branch_block, branch_expression) in [
            (then_block, &true_expression),
            (else_block, &false_expression),
        ] {
            let (values, current) =
                self.emit_conditional_branch_values(branch_expression, branch_block);
            assert!(
                values.len() == slots.len(),
                "a conditional branch yields one value per result slot"
            );
            for (index, value) in values.into_iter().enumerate() {
                let cast = value.cast(
                    crate::ast::Type::new(result_types[index]),
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
                slot.load(crate::ast::Type::new(result_types[index]), builder, &block)
                    .into_mlir(),
            );
        }
        (values, block)
    }

    /// Emits one branch of a tuple-valued conditional, expanding it to one value
    /// per result slot: a literal tuple yields each item's value; a
    /// tuple-returning call yields its full result list (like `return f();`); a
    /// nested conditional recurses.
    fn emit_conditional_branch_values(
        &self,
        branch: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> (
        Vec<crate::ast::Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        match branch {
            Expression::TupleExpression(tuple) => {
                let mut values = Vec::new();
                let mut current = block;
                for item in tuple.items().iter() {
                    let inner = item
                        .expression()
                        .expect("a multi-value conditional tuple element has an inner expression");
                    let BlockAnd { value, block: next } = inner.emit(self, current);
                    values.push(value);
                    current = next;
                }
                (values, current)
            }
            Expression::FunctionCallExpression(call) => {
                let (values, block) = self.emit_function_call_results(call, block);
                (
                    values.into_iter().map(crate::ast::Value::from).collect(),
                    block,
                )
            }
            Expression::ConditionalExpression(nested) => {
                let (values, block) = self.emit_conditional_tuple_values(nested, block);
                (
                    values.into_iter().map(crate::ast::Value::from).collect(),
                    block,
                )
            }
            other => unimplemented!(
                "multi-value conditional branch of this expression kind is not supported: {:?}",
                std::mem::discriminant(other)
            ),
        }
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

expression_emit!(ConditionalExpression; |node, context, block| {
    // A ternary whose branches are bare function names yields an *internal*
    // function pointer, but slang types it from the function's visibility —
    // a `Public` function as its return type — not the pointer type. The
    // branches emit `func_ref` values, so recover the internal-pointer type
    // from a branch when present; otherwise the conditional's own type is
    // authoritative, falling back to a branch's type when the binder leaves
    // the conditional untyped, rather than silently defaulting to `ui256`
    // (which masked the mismatch and `sol.cast`-ed a `func_ref` to integer).
    let result_type = context
        .bare_function_ref_type(&node.true_expression())
        .or_else(|| context.bare_function_ref_type(&node.false_expression()))
        .or_else(|| {
            crate::ast::Type::resolve_optional(node.get_type(), &context.state.builder)
        })
        .or_else(|| {
            crate::ast::Type::resolve_optional(
                node.true_expression().get_type(),
                &context.state.builder,
            )
        })
        .or_else(|| {
            crate::ast::Type::resolve_optional(
                node.false_expression().get_type(),
                &context.state.builder,
            )
        })
        .expect("a conditional resolves its type from itself or one of its branches");
    let condition = node.operand();
    let BlockAnd {
        value: condition_value,
        block,
    } = condition.emit(context, block);
    let condition_boolean = condition_value
        .is_nonzero(&context.state.builder, &block)
        .into_mlir();

    let result_slot = crate::ast::Pointer::stack_slot(
        crate::ast::Type::new(result_type),
        &context.state.builder,
        &block,
    );
    let (then_block, else_block) = sol_region_op!(
        &context.state.builder, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let true_expression = node.true_expression();
    let BlockAnd {
        value: then_value,
        block: then_end,
    } = (Toward {
        expression: &true_expression,
        target_type: result_type,
    })
    .emit(context, then_block);
    let then_cast = then_value.cast(
        crate::ast::Type::new(result_type),
        &context.state.builder,
        &then_end,
    );
    result_slot.store(then_cast, &context.state.builder, &then_end);
    sol_op_void!(&context.state.builder, &then_end, YieldOperation.ins(&[]));

    let false_expression = node.false_expression();
    let BlockAnd {
        value: else_value,
        block: else_end,
    } = (Toward {
        expression: &false_expression,
        target_type: result_type,
    })
    .emit(context, else_block);
    let else_cast = else_value.cast(
        crate::ast::Type::new(result_type),
        &context.state.builder,
        &else_end,
    );
    result_slot.store(else_cast, &context.state.builder, &else_end);
    sol_op_void!(&context.state.builder, &else_end, YieldOperation.ins(&[]));

    let result = result_slot.load(
        crate::ast::Type::new(result_type),
        &context.state.builder,
        &block,
    );

    BlockAnd { block, value: result }
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
        crate::ast::Type::resolve(&element_slang_type, LocationPolicy::ForceMemory, builder);
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
            crate::ast::Type::array(
                builder.context,
                solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                element_type,
                solx_utils::DataLocation::Memory,
            )
            .into_mlir()
        }
        _ => crate::ast::Type::resolve(&result_slang_type, LocationPolicy::ForceMemory, builder),
    };
    let element_values: Vec<_> = element_values
        .into_iter()
        .map(|value| {
            value
                .cast(crate::ast::Type::new(element_type), builder, &current)
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
