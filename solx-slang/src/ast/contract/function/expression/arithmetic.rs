//!
//! Arithmetic expression emission: additive, multiplicative, exponentiation, bitwise, and shift operations.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::AdditiveExpression;
use slang_solidity_v2::ast::BitwiseAndExpression;
use slang_solidity_v2::ast::BitwiseOrExpression;
use slang_solidity_v2::ast::BitwiseXorExpression;
use slang_solidity_v2::ast::ExponentiationExpression;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::MultiplicativeExpression;
use slang_solidity_v2::ast::ShiftExpression;

use solx_mlir::Type as AstType;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::emit::emit_expression::EmitExpression;

expression_emit!(AdditiveExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let operator = match node.operator() {
        ast::AdditiveExpressionOperator::Plus(_) => Operator::Add,
        ast::AdditiveExpressionOperator::Minus(_) => Operator::Subtract,
    };
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        operator,
        result_type,
        block,
    );
    BlockAnd { block, value }
});

expression_emit!(MultiplicativeExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let operator = match node.operator() {
        ast::MultiplicativeExpressionOperator::Asterisk(_) => Operator::Multiply,
        ast::MultiplicativeExpressionOperator::Percent(_) => Operator::Remainder,
        ast::MultiplicativeExpressionOperator::Slash(_) => Operator::Divide,
    };
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        operator,
        result_type,
        block,
    );
    BlockAnd { block, value }
});

expression_emit!(ExponentiationExpression; |node, context, block| {
    let target_type = context.resolve_slang_type(node.get_type());
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        Operator::Exponentiation,
        target_type,
        block,
    );
    BlockAnd { block, value }
});

expression_emit!(BitwiseAndExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        Operator::BitwiseAnd,
        result_type,
        block,
    );
    BlockAnd { block, value }
});

expression_emit!(BitwiseOrExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        Operator::BitwiseOr,
        result_type,
        block,
    );
    BlockAnd { block, value }
});

expression_emit!(BitwiseXorExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        Operator::BitwiseXor,
        result_type,
        block,
    );
    BlockAnd { block, value }
});

expression_emit!(ShiftExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let operator = match node.operator() {
        ast::ShiftExpressionOperator::GreaterThanGreaterThan(_) => Operator::ShiftRight,
        ast::ShiftExpressionOperator::GreaterThanGreaterThanGreaterThan(_) => Operator::ShiftRight,
        ast::ShiftExpressionOperator::LessThanLessThan(_) => Operator::ShiftLeft,
    };
    let BlockAnd { value, block } = context.emit_binary_op(
        &node.left_operand(),
        &node.right_operand(),
        operator,
        result_type,
        block,
    );
    BlockAnd { block, value }
});

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits a binary arithmetic Sol dialect operation.
    ///
    /// When `target_type` is `Some`, the left operand is cast to that type and
    /// the result has that type (matching solc's type-annotated MLIR output).
    /// When `None`, selects the wider operand type by bit width. Exponentiation
    /// and the shifts keep their right operand's own type: `sol.exp`/`sol.cexp`
    /// take an unsigned exponent alongside a possibly-signed base, and a shift
    /// amount is a plain integer, never the (possibly fixed-bytes) result type.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: Operator,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let BlockAnd { value: rhs, block } = right.emit(self, block);
        let BlockAnd { value: lhs, block } = left.emit(self, block);
        let result_type = target_type.unwrap_or_else(|| {
            let lhs_width = AstType::new(lhs.r#type()).integer_bit_width();
            let rhs_width = AstType::new(rhs.r#type()).integer_bit_width();
            if lhs_width >= rhs_width {
                lhs.r#type()
            } else {
                rhs.r#type()
            }
        });
        let lhs = TypeConversion::from_target_type(result_type, self.state).emit(
            lhs,
            self.state,
            &block,
        );
        let rhs = if matches!(
            operator,
            Operator::Exponentiation | Operator::ShiftLeft | Operator::ShiftRight
        ) {
            rhs
        } else {
            TypeConversion::from_target_type(result_type, self.state).emit(
                rhs,
                self.state,
                &block,
            )
        };
        let value = block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.mlir_context,
                self.state.location(),
                lhs,
                rhs,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into();
        BlockAnd { block, value }
    }
}
