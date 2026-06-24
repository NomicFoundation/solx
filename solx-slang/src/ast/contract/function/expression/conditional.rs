//!
//! Conditional, tuple, and array-literal expression emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::ArrayLitOperation;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for ConditionalExpression {
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Emits `cond ? a : b`, yielding one value per result (a scalar yields one, a tuple-valued
    /// conditional one per element). Both branches store into shared slots loaded after the `sol.if`.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let true_expression = self.true_expression();
        let false_expression = self.false_expression();

        // A tuple-valued conditional yields one value per element. A branch is a literal tuple (types
        // from its items) or a multi-value expression (types from the conditional's own tuple type).
        if let Some(SlangType::Tuple(tuple_type)) = self.get_type() {
            let result_types: Vec<Type<'context>> = match (&true_expression, &false_expression) {
                (Expression::TupleExpression(true_tuple), Expression::TupleExpression(_)) => {
                    let true_items: Vec<Expression> = true_tuple
                        .items()
                        .iter()
                        .filter_map(|item| item.expression())
                        .collect();
                    true_items
                        .iter()
                        .map(|item| {
                            AstType::resolve_optional(item.get_type(), &context.state.builder)
                                .expect("slang validated")
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
                        .expect("slang validated")
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
            let (then_block, else_block) = mlir_region_op!(builder, &block, IfOperation.cond(condition_boolean); then_region, else_region);

            for (branch_block, branch_expression) in [
                (then_block, &true_expression),
                (else_block, &false_expression),
            ] {
                // Expand the branch to one value per result slot (literal tuple, call result list, or nested conditional).
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
                    Expression::FunctionCallExpression(call) => {
                        let BlockAnd { value, block } = call.emit(context, branch_block);
                        (value, block)
                    }
                    Expression::ConditionalExpression(nested) => {
                        let BlockAnd { value, block } = nested.emit(context, branch_block);
                        (value, block)
                    }
                    other => unimplemented!(
                        "multi-value conditional branch of this expression kind is not supported: {:?}",
                        std::mem::discriminant(other)
                    ),
                };
                for (index, value) in values.into_iter().enumerate() {
                    let cast = AstValue::from(value).cast(
                        AstType::new(result_types[index]),
                        builder,
                        &current,
                    );
                    slots[index].store(cast, builder, &current);
                }
                mlir_op_void!(builder, &current, YieldOperation.ins(&[]));
            }

            let mut values = Vec::with_capacity(slots.len());
            for (index, &slot) in slots.iter().enumerate() {
                values.push(
                    slot.load(AstType::new(result_types[index]), builder, &block)
                        .into_mlir(),
                );
            }
            return BlockAnd {
                value: values,
                block,
            };
        }

        // A scalar ternary yields a single value. A branch of bare function names yields an internal
        // `func_ref`, but slang types it from the function's visibility, not the pointer type — so
        // recover the internal-pointer type from a branch when present, else use the conditional's own type.
        let func_ref_type = |expression: &Expression| {
            let Expression::Identifier(identifier) = expression else {
                return None;
            };
            let Definition::Function(function_definition) = identifier.resolve_to_definition()?
            else {
                return None;
            };
            let function = context
                .state
                .resolve_function(function_definition.node_id());
            Some(function.func_ref_type(&context.state.builder).into_mlir())
        };
        let result_type = func_ref_type(&true_expression)
            .or_else(|| func_ref_type(&false_expression))
            .or_else(|| AstType::resolve_optional(self.get_type(), &context.state.builder))
            .expect("slang validated");
        let BlockAnd {
            value: condition_value,
            block,
        } = self.operand().emit(context, block);
        let condition_boolean = condition_value
            .is_nonzero(&context.state.builder, &block)
            .into_mlir();

        let result_slot =
            Pointer::stack_slot(AstType::new(result_type), &context.state.builder, &block);
        let (then_block, else_block) = mlir_region_op!(
            &context.state.builder, &block,
            IfOperation.cond(condition_boolean); then_region, else_region
        );

        // `emit_as` already routes a string literal to its target representation and
        // casts the value to `result_type`, so both branches share one body.
        for (branch_block, branch_expression) in [
            (then_block, &true_expression),
            (else_block, &false_expression),
        ] {
            let BlockAnd {
                value: branch_value,
                block: branch_end,
            } = branch_expression.emit_as(result_type, context, branch_block);
            result_slot.store(branch_value, &context.state.builder, &branch_end);
            mlir_op_void!(&context.state.builder, &branch_end, YieldOperation.ins(&[]));
        }

        let result = result_slot.load(AstType::new(result_type), &context.state.builder, &block);
        BlockAnd {
            value: vec![result.into_mlir()],
            block,
        }
    }
}

expression_emit!(TupleExpression; |node, context, block| {
    let items = node.items();
    // TODO: support multi-value tuples (e.g. tuple deconstruction)
    let item = items.iter().next().expect("slang validated");
    let inner = item
        .expression()
        .expect("slang validated");
    inner.emit(context, block)
});

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
    let builder = &context.state.builder;
    // An array literal is always a memory aggregate, so resolve element and result types in their
    // memory representation — the per-element coercion is then a `data_loc_cast` into memory (matching solc).
    let declared_element_type =
        AstType::resolve(&element_slang_type, LocationPolicy::ForceMemory, builder);
    // Emit element values before fixing the element type: for a function-pointer array literal the
    // emitted values are authoritative (slang types the literal from visibility, which can disagree),
    // so adopt the value's function-ref type when it differs and rebuild the array type to match.
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
    let value: Value<'context, 'block> = mlir_op!(
        builder,
        &current,
        ArrayLitOperation.ins(&element_values).addr(array_type)
    );
    BlockAnd { block: current, value: value.into() }
});
