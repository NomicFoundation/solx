//!
//! Function definition lowering to Sol dialect MLIR.
//!

/// Expression lowering to MLIR SSA values.
pub(crate) mod expression;
/// Sol dialect state mutability encoding.
pub mod state_mutability;
/// Statement lowering to MLIR operations.
pub(crate) mod statement;

use melior::ir::BlockLike;
use slang_solidity::backend::ir::ast::FunctionDefinition;
use slang_solidity::backend::ir::ast::FunctionVisibility;

use solx_mlir::Environment;

use crate::slang::codegen::MlirContext;
use crate::slang::codegen::selector::SelectorComputer;
use crate::slang::codegen::types::TypeMapper;

use self::expression::ExpressionEmitter;
use self::state_mutability::StateMutability;
use self::statement::StatementEmitter;

/// Lowers a Solidity function definition to a `sol.func` operation.
pub(crate) struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state MlirContext<'context>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub(crate) fn new(state: &'state MlirContext<'context>) -> Self {
        Self { state }
    }

    /// Emits a `sol.func` for the given function definition into the given
    /// contract body block.
    ///
    /// # Errors
    ///
    /// Returns an error if the function body contains unsupported statements.
    ///
    /// # Panics
    ///
    /// Panics if an entry block is not attached to a region, which is
    /// unreachable because `emit_sol_func` always creates a region.
    pub(crate) fn emit_sol(
        &self,
        function: &FunctionDefinition,
        contract_body: &melior::ir::BlockRef<'context, '_>,
    ) -> anyhow::Result<String> {
        let name = function
            .name()
            .map(|id| id.name())
            .unwrap_or_else(|| "unnamed".to_owned());

        let parameters = function.parameters();
        let parameter_types: Vec<String> = parameters
            .iter()
            .map(|p| TypeMapper::canonical_type(&p.type_name()))
            .collect::<anyhow::Result<_>>()?;
        let mlir_name = format!("solx.fn.{name}({})", parameter_types.join(","));

        let i256 = self.state.i256();

        let has_returns = function
            .returns()
            .is_some_and(|returns| !returns.is_empty());

        let mlir_parameter_types: Vec<melior::ir::Type<'context>> =
            parameter_types.iter().map(|_| i256).collect();
        let result_types: Vec<melior::ir::Type<'context>> =
            if has_returns { vec![i256] } else { vec![] };

        // Compute selector for external/public functions.
        let is_dispatched = matches!(
            function.visibility(),
            FunctionVisibility::External | FunctionVisibility::Public
        );
        let selector = if is_dispatched {
            let (selector_bytes, _signature) = SelectorComputer::compute(function)?;
            Some(u32::from_be_bytes(selector_bytes))
        } else {
            None
        };

        let state_mutability = Self::map_state_mutability(function);

        let function_entry_block = self.state.emit_sol_func(
            &mlir_name,
            &mlir_parameter_types,
            &result_types,
            selector,
            state_mutability as u32,
            contract_body,
        );

        let mut environment = Environment::new();

        // Create allocas for parameters and bind to environment.
        for (i, parameter) in parameters.iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_value: melior::ir::Value<'context, '_> =
                function_entry_block.argument(i)?.into();

            let region = function_entry_block
                .parent_region()
                .expect("entry block belongs to a region");
            let expression_emitter = ExpressionEmitter::new(self.state, &environment, &region);
            let pointer = expression_emitter.emit_alloca(&function_entry_block);
            expression_emitter.emit_store(parameter_value, pointer, &function_entry_block);

            if TypeMapper::is_signed(&parameter.type_name()) {
                environment.mark_signed(&parameter_name);
            }
            environment.define_variable(parameter_name, pointer);
        }

        if let Some(ref body) = function.body() {
            let region = function_entry_block
                .parent_region()
                .expect("entry block belongs to a region");
            let mut current_block = function_entry_block;
            let mut terminated = false;
            for statement in body.statements().iter() {
                let mut emitter = StatementEmitter::new(self.state, &mut environment, &region);
                match emitter.emit(&statement, current_block)? {
                    Some(next) => current_block = next,
                    None => {
                        terminated = true;
                        break;
                    }
                }
            }

            if !terminated {
                self.emit_default_return(has_returns, &current_block);
            }
        } else {
            self.emit_default_return(has_returns, &function_entry_block);
        }

        Ok(mlir_name)
    }

    /// Maps Solidity function state mutability to Sol dialect `StateMutability`.
    fn map_state_mutability(function: &FunctionDefinition) -> StateMutability {
        use slang_solidity::backend::ir::ast::FunctionMutability;
        match function.mutability() {
            FunctionMutability::Pure => StateMutability::Pure,
            FunctionMutability::View => StateMutability::View,
            FunctionMutability::Payable => StateMutability::Payable,
            FunctionMutability::NonPayable => StateMutability::NonPayable,
        }
    }

    /// Emits a default `sol.return` if the block lacks a terminator.
    fn emit_default_return(&self, has_returns: bool, block: &melior::ir::BlockRef<'context, '_>) {
        if block.terminator().is_some() {
            return;
        }
        if has_returns {
            let zero = self.state.emit_sol_constant(0, block);
            self.state.emit_sol_return(&[zero], block);
        } else {
            self.state.emit_sol_return(&[], block);
        }
    }
}
