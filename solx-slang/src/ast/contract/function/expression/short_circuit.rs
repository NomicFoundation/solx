//!
//! Short-circuit logical expression emission: `&&` and `||`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AndExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::OrExpression;

use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(AndExpression; |node, context, block| {
    context.emit_and(&node.left_operand(), &node.right_operand(), block)
});

expression_emit!(OrExpression; |node, context, block| {
    context.emit_or(&node.left_operand(), &node.right_operand(), block)
});

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits short-circuit `&&` using `sol.if` with an `i1` alloca.
    ///
    /// Matches solc's pattern: allocate a boolean result variable, default to
    /// `false`, and only evaluate the RHS when the LHS is true.
    pub fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let BlockAnd { value: lhs, block } = left.emit(self, block);
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type =
            AstType::signless(self.state.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        let result_ptr = Pointer::stack(AstType::new(i1_type), self.state, &block).into_mlir();
        let false_value = AstValue::boolean(false, self.state, &block).into_mlir();
        Pointer::new(result_ptr).store(AstValue::new(false_value), self.state, &block);

        let (then_block, else_block) =
            mlir_region_op!(self.state, &block, IfOperation.cond(lhs_bool); then_region, else_region);

        let BlockAnd {
            value: rhs,
            block: then_end,
        } = right.emit(self, then_block);
        let rhs_bool = self.emit_is_nonzero(rhs, &then_end);
        Pointer::new(result_ptr).store(AstValue::new(rhs_bool), self.state, &then_end);
        mlir_op_void!(self.state, &then_end, YieldOperation.ins(&[]));

        mlir_op_void!(self.state, &else_block, YieldOperation.ins(&[]));

        let value =
            Pointer::new(result_ptr).load(AstType::new(i1_type), self.state, &block).into_mlir();
        BlockAnd { block, value }
    }

    /// Emits short-circuit `||` using `sol.if` with an `i1` alloca.
    ///
    /// Matches solc's pattern: allocate a boolean result variable, default to
    /// `true`, and only evaluate the RHS when the LHS is false.
    pub fn emit_or(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let BlockAnd { value: lhs, block } = left.emit(self, block);
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type =
            AstType::signless(self.state.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        let result_ptr = Pointer::stack(AstType::new(i1_type), self.state, &block).into_mlir();
        let true_value = AstValue::boolean(true, self.state, &block).into_mlir();
        Pointer::new(result_ptr).store(AstValue::new(true_value), self.state, &block);

        let (then_block, else_block) =
            mlir_region_op!(self.state, &block, IfOperation.cond(lhs_bool); then_region, else_region);

        mlir_op_void!(self.state, &then_block, YieldOperation.ins(&[]));

        let BlockAnd {
            value: rhs,
            block: else_end,
        } = right.emit(self, else_block);
        let rhs_bool = self.emit_is_nonzero(rhs, &else_end);
        Pointer::new(result_ptr).store(AstValue::new(rhs_bool), self.state, &else_end);
        mlir_op_void!(self.state, &else_end, YieldOperation.ins(&[]));

        let value =
            Pointer::new(result_ptr).load(AstType::new(i1_type), self.state, &block).into_mlir();
        BlockAnd { block, value }
    }
}
