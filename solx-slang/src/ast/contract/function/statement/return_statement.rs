//!
//! Return statement lowering to `sol.return`.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::ReturnStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

use super::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `return` statement to a `sol.return`, terminating the block.
    ///
    /// A bare `return;` emits an operand-less `sol.return`. Otherwise the
    /// returned value is cast to the declared return type before becoming the
    /// `sol.return` operand. Returns `None` because control flow terminates.
    pub(super) fn emit_return(
        &self,
        return_statement: &ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(expression) = return_statement.expression() else {
            self.state.builder.emit_sol_return(&[], &block);
            return Ok(None);
        };

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );
        let (value, block) = emitter.emit_value(&expression, block)?;

        let return_type = self.return_types[0];
        let cast = TypeConversion::from_target_type(return_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        self.state.builder.emit_sol_return(&[cast], &block);
        Ok(None)
    }
}
