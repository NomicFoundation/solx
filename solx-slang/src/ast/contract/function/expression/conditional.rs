//!
//! Conditional (ternary) expression emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_values::EmitValues;

expression_emit!(ConditionalExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type()).unwrap_or_else(|| {
        AstType::unsigned(context.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
    });
    let condition = node.operand();
    let BlockAnd {
        value: condition_value,
        block,
    } = condition.emit(context, block);
    let condition_boolean = context.emit_is_nonzero(condition_value, &block);

    let result_slot = Pointer::stack(AstType::new(result_type), context.state, &block);
    let (then_block, else_block) = mlir_region_op!(
        context.state, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let true_expression = node.true_expression();
    let BlockAnd {
        value: then_value,
        block: then_end,
    } = true_expression.emit(context, then_block);
    let then_cast = TypeConversion::from_target_type(result_type, context.state)
        .emit(then_value, context.state, &then_end);
    result_slot.store(AstValue::new(then_cast), context.state, &then_end);
    mlir_op_void!(context.state, &then_end, YieldOperation.ins(&[]));

    let false_expression = node.false_expression();
    let BlockAnd {
        value: else_value,
        block: else_end,
    } = false_expression.emit(context, else_block);
    let else_cast = TypeConversion::from_target_type(result_type, context.state)
        .emit(else_value, context.state, &else_end);
    result_slot.store(AstValue::new(else_cast), context.state, &else_end);
    mlir_op_void!(context.state, &else_end, YieldOperation.ins(&[]));

    let value = result_slot
        .load(AstType::new(result_type), context.state, &block)
        .into_mlir();
    BlockAnd { block, value }
});

impl<'context: 'block, 'block> EmitValues<'context, 'block> for ConditionalExpression {
    /// Emits a tuple-valued `cond ? (a, b) : (c, d)`, yielding one value per element. Each result
    /// slot is stored in both `sol.if` branches from the branch's expanded values, then loaded after.
    fn emit_values<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let true_expression = self.true_expression();
        let false_expression = self.false_expression();

        let result_types: Vec<Type<'context>> =
            match (&true_expression, &false_expression, self.get_type()) {
                (Expression::TupleExpression(true_tuple), Expression::TupleExpression(_), _) => {
                    true_tuple
                        .items()
                        .iter()
                        .map(|item| {
                            let element = item.expression().expect("slang validates tuple element");
                            context
                                .resolve_slang_type(element.get_type())
                                .expect("slang types every tuple element")
                        })
                        .collect()
                }
                (_, _, Some(SlangType::Tuple(tuple_type))) => tuple_type
                    .types()
                    .into_iter()
                    .map(|element_type| {
                        context
                            .resolve_slang_type(Some(element_type))
                            .expect("slang types every tuple element")
                    })
                    .collect(),
                _ => unreachable!("a multi-valued conditional is tuple-typed"),
            };

        let condition = self.operand();
        let BlockAnd {
            value: condition_value,
            block,
        } = condition.emit(context, block);
        let condition_boolean = context.emit_is_nonzero(condition_value, &block);

        let result_slots: Vec<Pointer<'context, 'block>> = result_types
            .iter()
            .map(|&result_type| Pointer::stack(AstType::new(result_type), context.state, &block))
            .collect();
        let (then_block, else_block) = mlir_region_op!(
            context.state, &block,
            IfOperation.cond(condition_boolean); then_region, else_region
        );

        for (branch_block, branch_expression) in [
            (then_block, &true_expression),
            (else_block, &false_expression),
        ] {
            let BlockAnd {
                value: branch_values,
                block: branch_end,
            } = branch_expression.emit_values(context, branch_block);
            for (index, branch_value) in branch_values.into_iter().enumerate() {
                let cast = TypeConversion::from_target_type(result_types[index], context.state)
                    .emit(branch_value, context.state, &branch_end);
                result_slots[index].store(AstValue::new(cast), context.state, &branch_end);
            }
            mlir_op_void!(context.state, &branch_end, YieldOperation.ins(&[]));
        }

        let value = result_types
            .iter()
            .zip(result_slots)
            .map(|(&result_type, slot)| {
                slot.load(AstType::new(result_type), context.state, &block)
                    .into_mlir()
            })
            .collect();
        BlockAnd { block, value }
    }
}
