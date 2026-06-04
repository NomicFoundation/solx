//!
//! Return statement lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ReturnStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Emits a return statement.
    ///
    /// A multi-element tuple in return position is unpacked into one value per
    /// declared return slot; any other expression yields a single value. Each
    /// value is cast to its declared return type before being emitted as a
    /// `sol.return` operand.
    pub fn emit_return(
        &mut self,
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

        let (values, block) = if let Expression::TupleExpression(tuple) = &expression
            && tuple.items().len() > 1
        {
            let items = tuple.items();
            let mut values = Vec::with_capacity(items.len());
            let mut current = block;
            for item in items.iter() {
                let inner = item
                    .expression()
                    .expect("a return tuple element wraps an expression");
                let (value, next) = emitter.emit_value(&inner, current)?;
                values.push(value);
                current = next;
            }
            (values, current)
        } else {
            let (value, block) = emitter.emit_value(&expression, block)?;
            (vec![value], block)
        };

        let cast_values: Vec<_> = values
            .into_iter()
            .zip(self.return_types.iter())
            .map(|(value, &return_type)| {
                TypeConversion::from_target_type(return_type, &self.state.builder).emit(
                    value,
                    &self.state.builder,
                    &block,
                )
            })
            .collect();

        self.state.builder.emit_sol_return(&cast_values, &block);
        Ok(None)
    }
}
