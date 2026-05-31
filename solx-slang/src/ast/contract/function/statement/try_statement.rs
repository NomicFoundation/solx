//! Try-catch statement lowering.
//!
//! Experimental: emits only the success path of `try expr returns (...) { body }
//! catch { ... }`. Catch clauses are ignored — a real implementation requires
//! a `sol.try` op with exception regions, which is out of scope for the
//! Slang frontend bring-up. Tests that rely on the catch path being taken
//! will fail at runtime; tests where the try expression succeeds should pass.

use melior::ir::BlockRef;

use slang_solidity_v2::ast::TryStatement;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::statement::StatementEmitter;

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Lowers a `try` statement by emitting only the success path.
    pub fn emit_try(
        &mut self,
        try_statement: &TryStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let expression = try_statement.expression();
        let returns = try_statement.returns();

        let emitter = ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.checked,
        );

        let (value, current_block) = emitter.emit(&expression, block)?;

        if let Some(parameters) = returns
            && let Some(value) = value
        {
            let parameter_list: Vec<_> = parameters.iter().collect();
            // Single-return case: bind the call result to the first named
            // return parameter, if it has a name. Multi-return tuple
            // destructuring is not yet handled.
            if let Some(parameter) = parameter_list.first()
                && let Some(name_identifier) = parameter.name()
            {
                let name = name_identifier.name();
                let parameter_type = parameter
                    .get_type()
                    .map(|slang_type| {
                        TypeConversion::resolve_slang_type(
                            &slang_type,
                            None,
                            &self.state.builder,
                        )
                    })
                    .unwrap_or_else(|| self.state.builder.types.ui256);
                let cast = TypeConversion::from_target_type(parameter_type, &self.state.builder)
                    .emit(value, &self.state.builder, &current_block);
                let pointer = self
                    .state
                    .builder
                    .emit_sol_alloca(parameter_type, &current_block);
                self.state.builder.emit_sol_store(cast, pointer, &current_block);
                self.environment.define_variable(name, pointer, parameter_type);
            }
        }

        self.emit_block(try_statement.body().statements(), current_block)
    }
}
