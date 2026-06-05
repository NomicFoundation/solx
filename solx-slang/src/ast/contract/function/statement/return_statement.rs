//!
//! Return statement lowering to `sol.return`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::ReturnStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `return` statement to a `sol.return`, terminating control flow.
    ///
    /// A bare `return;` emits an operand-less `sol.return`. A tuple
    /// (`return (a, b);`) returns one value per component, and a single call
    /// returning the whole tuple (`return f();`) returns one value per declared
    /// return; any other expression returns a single value. Each returned value
    /// is cast to its declared return type.
    pub fn emit_return(
        &self,
        return_statement: &ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(expression) = return_statement.expression() else {
            // A bare `return;` returns the current values of the named returns,
            // loaded from their slots (a typed zero for an unnamed slot) — the
            // same values the fall-off-the-end epilogue yields, and what a
            // modifier stage threads on to its caller.
            let mut values: Vec<Value<'context, 'block>> =
                Vec::with_capacity(self.return_types.len());
            for (index, &return_type) in self.return_types.iter().enumerate() {
                let value = match self.return_slots.get(index).copied().flatten() {
                    Some(pointer) => {
                        self.state
                            .builder
                            .emit_sol_load(pointer, return_type, &block)?
                    }
                    None => self.state.builder.emit_sol_constant(0, return_type, &block),
                };
                values.push(value);
            }
            self.state.builder.emit_sol_return(&values, &block);
            return Ok(None);
        };

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        // A single call returning the whole tuple (`return f();` /
        // `return addr.delegatecall(d);`) yields one value per declared return,
        // so its `sol.return` arity matches the function. Every other form (an
        // explicit `(a, b)` tuple, or a scalar) resolves component-wise.
        let (values, block) = if self.return_types.len() > 1
            && let Expression::FunctionCallExpression(call) = &expression
        {
            CallEmitter::new(&emitter).emit_function_call_results(call, block)?
        } else {
            emitter.emit_component_values(&expression, block)?
        };

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
