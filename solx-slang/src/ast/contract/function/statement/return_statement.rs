//!
//! Return statement lowering to `sol.return`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ReturnStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `return` statement to a `sol.return`, terminating control flow.
    ///
    /// A bare `return;` emits an operand-less `sol.return`. A tuple
    /// (`return (a, b);`) returns one value per component; any other expression
    /// returns a single value. Each returned value is cast to its declared
    /// return type.
    pub fn emit_return(
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
        let (values, block) = emitter.emit_returned_values(&expression, block)?;

        let cast_values: Vec<Value<'context, 'block>> = values
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

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Evaluates a returned expression into one value per declared return.
    ///
    /// A tuple of more than one component yields its component values; a
    /// single-component tuple is just a parenthesized expression, and any other
    /// expression yields a single value.
    fn emit_returned_values(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Expression::TupleExpression(tuple) = expression else {
            let (value, block) = self.emit_value(expression, block)?;
            return Ok((vec![value], block));
        };
        let items = tuple.items();
        if items.len() <= 1 {
            let (value, block) = self.emit_value(expression, block)?;
            return Ok((vec![value], block));
        }

        let mut values = Vec::with_capacity(items.len());
        let mut block = block;
        for item in items.iter() {
            let component = item
                .expression()
                .expect("a multi-component return tuple has no empty components");
            let (value, next_block) = self.emit_value(&component, block)?;
            values.push(value);
            block = next_block;
        }
        Ok((values, block))
    }
}
