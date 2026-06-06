//!
//! Comparison expression lowering: equality and inequality (`sol.cmp`).
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Expression;
use solx_mlir::CmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a `sol.cmp` comparison.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        predicate: CmpPredicate,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let (rhs, block) = self.emit_value(right, block)?;
        let common_type = if lhs.r#type() == rhs.r#type() {
            lhs.r#type()
        } else {
            self.state.builder.types.ui256
        };
        let lhs = TypeConversion::from_target_type(common_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs = TypeConversion::from_target_type(common_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
        Ok((comparison, block))
    }
}
