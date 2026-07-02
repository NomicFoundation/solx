//!
//! Comparison expression emission: equality and inequality, reconciling the operand types first.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;

use solx_mlir::CmpPredicate;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(EqualityExpression; |node, context, block| {
    let predicate = match node.operator() {
        ast::EqualityExpressionOperator::BangEqual(_) => CmpPredicate::Ne,
        ast::EqualityExpressionOperator::EqualEqual(_) => CmpPredicate::Eq,
    };
    context.emit_comparison(&node.left_operand(), &node.right_operand(), predicate, block)
});

expression_emit!(InequalityExpression; |node, context, block| {
    let predicate = match node.operator() {
        ast::InequalityExpressionOperator::GreaterThan(_) => CmpPredicate::Gt,
        ast::InequalityExpressionOperator::GreaterThanEqual(_) => CmpPredicate::Ge,
        ast::InequalityExpressionOperator::LessThan(_) => CmpPredicate::Lt,
        ast::InequalityExpressionOperator::LessThanEqual(_) => CmpPredicate::Le,
    };
    context.emit_comparison(&node.left_operand(), &node.right_operand(), predicate, block)
});

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a `sol.cmp` comparison, reconciling operands of differing types first.
    ///
    /// Two different-width fixed-bytes operands are widened left and compared as fixed-bytes;
    /// two integers of differing widths widen to the wider operand type, comparing as signed when
    /// either side is signed so a negative is not read as a large unsigned value.
    pub fn emit_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        predicate: CmpPredicate,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        if let Some(function_id) = self.user_defined_operator(left, predicate.into()) {
            let BlockAnd { value: lhs, block } = left.emit(self, block);
            let BlockAnd { value: rhs, block } = right.emit(self, block);
            let value = self.emit_operator_call(function_id, vec![lhs, rhs], &block);
            return BlockAnd { block, value };
        }
        let BlockAnd { value: lhs, block } = left.emit(self, block);
        let BlockAnd { value: rhs, block } = right.emit(self, block);
        let lhs_type = lhs.r#type();
        let rhs_type = rhs.r#type();
        let value = if lhs_type == rhs_type {
            AstValue::new(lhs)
                .compare(AstValue::new(rhs), predicate, self.state, &block)
                .into_mlir()
        } else if let Some(width) = self
            .fixed_bytes_or_byte_width(lhs_type)
            .into_iter()
            .chain(self.fixed_bytes_or_byte_width(rhs_type))
            .max()
        {
            let common = AstType::fixed_bytes(self.state.mlir_context, width);
            let lhs = AstValue::new(lhs).bytes_cast(common, self.state, &block);
            let rhs = AstValue::new(rhs).bytes_cast(common, self.state, &block);
            lhs.compare(rhs, predicate, self.state, &block).into_mlir()
        } else {
            let common = if AstType::new(lhs_type).integer_bit_width()
                >= AstType::new(rhs_type).integer_bit_width()
            {
                lhs_type
            } else {
                rhs_type
            };
            let lhs = AstValue::new(lhs).cast(AstType::new(common), self.state, &block);
            let rhs = AstValue::new(rhs).cast(AstType::new(common), self.state, &block);
            lhs.compare(rhs, predicate, self.state, &block).into_mlir()
        };
        BlockAnd { block, value }
    }
}
