//!
//! Conditional (ternary) expression emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_for_effect::EmitForEffect;
use crate::ast::emit::emit_values::EmitValues;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for ConditionalExpression {
    type Output = BlockAnd<'context, 'block, Vec<Value<'context, 'block>>>;

    /// Emits `cond ? a : b`, yielding one value per result. Both branches store into shared slots
    /// loaded after the `sol.if`.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let true_expression = self.true_expression().unwrap_parentheses();
        let false_expression = self.false_expression().unwrap_parentheses();

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
                            AstType::resolve_optional(item.get_type(), context.state)
                                .expect("slang validated")
                        })
                        .collect()
                }
                _ => tuple_type
                    .types()
                    .iter()
                    .map(|element_type| {
                        AstType::resolve_optional(Some(element_type.clone()), context.state)
                            .expect("slang validated")
                    })
                    .collect(),
            };

            let state = context.state;
            let BlockAnd {
                value: condition_value,
                block,
            } = self.operand().emit(context, block);
            let condition_boolean = condition_value.is_nonzero(state, &block).into_mlir();
            let slots: Vec<Pointer<'context, 'block>> = result_types
                .iter()
                .map(|&result_type| Pointer::stack(AstType::new(result_type), state, &block))
                .collect();
            let (then_block, else_block) = mlir_region_op!(state, &block, IfOperation.cond(condition_boolean); then_region, else_region);

            for (branch_block, branch_expression) in [
                (then_block, &true_expression),
                (else_block, &false_expression),
            ] {
                let BlockAnd {
                    value: values,
                    block: current,
                } = branch_expression.emit_values(context, branch_block);
                for (index, value) in values.into_iter().enumerate() {
                    let cast = AstValue::from(value).cast(
                        AstType::new(result_types[index]),
                        state,
                        &current,
                    );
                    slots[index].store(cast, state, &current);
                }
                mlir_op_void!(state, &current, YieldOperation.ins(&[]));
            }

            let mut values = Vec::with_capacity(slots.len());
            for (index, &slot) in slots.iter().enumerate() {
                values.push(
                    slot.load(AstType::new(result_types[index]), state, &block)
                        .into_mlir(),
                );
            }
            return BlockAnd {
                value: values,
                block,
            };
        }

        let func_ref_type = |expression: &Expression| {
            let definition = match expression {
                Expression::Identifier(identifier) => identifier.resolve_to_definition()?,
                Expression::MemberAccessExpression(access) => {
                    let Expression::Identifier(operand) = access.operand() else {
                        return None;
                    };
                    if !matches!(
                        operand.resolve_to_definition(),
                        Some(Definition::Contract(_))
                    ) {
                        return None;
                    }
                    access.member().resolve_to_definition()?
                }
                _ => return None,
            };
            let Definition::Function(function_definition) = definition else {
                return None;
            };
            let function = context
                .state
                .resolve_function(function_definition.node_id());
            Some(function.func_ref_type(context.state).into_mlir())
        };
        let result_type = func_ref_type(&true_expression)
            .or_else(|| func_ref_type(&false_expression))
            .or_else(|| AstType::resolve_optional(self.get_type(), context.state))
            .expect("slang validated");
        let BlockAnd {
            value: condition_value,
            block,
        } = self.operand().emit(context, block);
        let condition_boolean = condition_value
            .is_nonzero(context.state, &block)
            .into_mlir();

        let result_slot = Pointer::stack(AstType::new(result_type), context.state, &block);
        let (then_block, else_block) = mlir_region_op!(
            context.state, &block,
            IfOperation.cond(condition_boolean); then_region, else_region
        );

        for (branch_block, branch_expression) in [
            (then_block, &true_expression),
            (else_block, &false_expression),
        ] {
            let BlockAnd {
                value: branch_value,
                block: branch_end,
            } = branch_expression.emit_as(result_type, context, branch_block);
            result_slot.store(branch_value, context.state, &branch_end);
            mlir_op_void!(context.state, &branch_end, YieldOperation.ins(&[]));
        }

        BlockAnd {
            value: vec![
                result_slot
                    .load(AstType::new(result_type), context.state, &block)
                    .into_mlir(),
            ],
            block,
        }
    }
}

impl<'context: 'block, 'block> EmitForEffect<'context, 'block> for ConditionalExpression {
    /// Emits a void-typed `cond ? a : b` for effect: one branch runs per the condition, with no result.
    fn emit_for_effect<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let true_expression = self.true_expression().unwrap_parentheses();
        let false_expression = self.false_expression().unwrap_parentheses();
        let BlockAnd {
            value: condition_value,
            block,
        } = self.operand().emit(context, block);
        let condition_boolean = condition_value
            .is_nonzero(context.state, &block)
            .into_mlir();
        let (then_block, else_block) = mlir_region_op!(
            context.state, &block,
            IfOperation.cond(condition_boolean); then_region, else_region
        );

        for (branch_block, branch_expression) in [
            (then_block, &true_expression),
            (else_block, &false_expression),
        ] {
            let branch_end = branch_expression.emit_for_effect(context, branch_block);
            mlir_op_void!(context.state, &branch_end, YieldOperation.ins(&[]));
        }

        block
    }
}
