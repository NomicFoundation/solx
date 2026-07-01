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
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
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
    /// Emits a `sol.cmp` comparison.
    pub fn emit_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        predicate: CmpPredicate,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let BlockAnd { value: lhs, block } = left.emit(self, block);
        let BlockAnd { value: rhs, block } = right.emit(self, block);
        let common_type = if lhs.r#type() == rhs.r#type() {
            lhs.r#type()
        } else {
            AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
        };
        let lhs = TypeConversion::from_target_type(common_type, self.state).emit(
            lhs,
            self.state,
            &block,
        );
        let rhs = TypeConversion::from_target_type(common_type, self.state).emit(
            rhs,
            self.state,
            &block,
        );
        let value = AstValue::new(lhs)
            .compare(AstValue::new(rhs), predicate, self.state, &block)
            .into_mlir();
        BlockAnd { block, value }
    }
}
