//!
//! Function definition lowering to MLIR.
//!

use melior::dialect::llvm;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;

use slang_solidity::backend::ir::ast::FunctionDefinition;

use solx_mlir::Environment;

use crate::codegen::MlirContext;
use crate::codegen::expression::ExpressionEmitter;
use crate::codegen::statement::StatementEmitter;
use crate::codegen::types::TypeMapper;

/// Lowers a Solidity function definition to an MLIR `llvm.func` operation.
pub struct FunctionEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state MlirContext<'context>,
}

impl<'state, 'context> FunctionEmitter<'state, 'context> {
    /// Creates a new function emitter.
    pub(crate) fn new(state: &'state MlirContext<'context>) -> Self {
        Self { state }
    }

    /// Emits an `llvm.func` for the given function definition.
    ///
    /// The function is appended to the module body. Returns the mangled
    /// name used for the MLIR function symbol.
    ///
    /// # Errors
    ///
    /// Returns an error if the function body contains unsupported statements.
    pub(crate) fn emit(&self, function: &FunctionDefinition) -> anyhow::Result<String> {
        let name = function
            .name()
            .map(|id| id.name())
            .unwrap_or_else(|| "unnamed".to_owned());

        let parameters = function.parameters();
        let parameter_types: Vec<String> = parameters
            .iter()
            .map(|p| TypeMapper::canonical_type(&p.type_name()))
            .collect();
        let mlir_name = format!("solx.fn.{name}({})", parameter_types.join(","));

        let context = self.state.context();
        let location = self.state.location();
        let i256 = self.state.i256();

        let has_returns = function
            .returns()
            .is_some_and(|returns| !returns.is_empty());

        let mlir_parameter_types: Vec<melior::ir::Type<'context>> =
            parameter_types.iter().map(|_| i256).collect();

        let function_type = llvm::r#type::function(
            if has_returns {
                i256
            } else {
                llvm::r#type::void(context)
            },
            &mlir_parameter_types,
            false,
        );

        let region = Region::new();
        let block_arguments: Vec<(melior::ir::Type<'context>, melior::ir::Location<'context>)> =
            mlir_parameter_types.iter().map(|t| (*t, location)).collect();
        let entry_block = region.append_block(Block::new(&block_arguments));

        let mut environment = Environment::new();

        // Create allocas for parameters and bind to environment.
        for (i, parameter) in parameters.iter().enumerate() {
            let parameter_name = parameter
                .name()
                .map(|id| id.name())
                .unwrap_or_else(|| "_".to_owned());
            let parameter_value: melior::ir::Value<'context, '_> = entry_block.argument(i)?.into();

            let expression_emitter = ExpressionEmitter::new(self.state, &environment, &region);
            let ptr = expression_emitter.emit_alloca(&entry_block);
            expression_emitter.emit_store(parameter_value, ptr, &entry_block);

            if TypeMapper::is_signed(&parameter.type_name()) {
                environment.mark_signed(&parameter_name);
            }
            environment.define_variable(parameter_name, ptr);
        }

        if let Some(ref body) = function.body() {
            let mut current_block = entry_block;
            for statement in body.statements().iter() {
                let mut emitter = StatementEmitter::new(self.state, &mut environment, &region);
                match emitter.emit(&statement, current_block)? {
                    Some(next) => current_block = next,
                    None => {
                        // Control flow terminated (return/break/continue).
                        // Remaining statements are dead code.
                        current_block =
                            region.append_block(Block::new(&[]));
                        break;
                    }
                }
            }

            // Add default terminator if block is not yet terminated.
            self.emit_default_return(has_returns, &current_block);
        } else {
            self.emit_default_return(has_returns, &entry_block);
        }

        // Ensure every block has a terminator (dead blocks from control flow
        // may be empty).
        Self::terminate_empty_blocks(&region, has_returns, self.state);

        let function_operation = llvm::func(
            context,
            StringAttribute::new(context, &mlir_name),
            TypeAttribute::new(function_type),
            region,
            &[],
            location,
        );

        self.state.body().append_operation(function_operation);

        Ok(mlir_name)
    }

    /// Adds `llvm.unreachable` to any block that lacks a terminator.
    ///
    /// Dead blocks can appear from control flow constructs (e.g. `for` loop
    /// exit/iter blocks when the body always returns). MLIR requires every
    /// block to end with a terminator.
    fn terminate_empty_blocks(region: &Region<'context>, has_returns: bool, state: &MlirContext<'context>) {
        let location = state.location();
        let mut maybe_block = region.first_block();
        while let Some(block) = maybe_block {
            let needs_terminator = block.terminator().is_none();
            if needs_terminator {
                if has_returns {
                    let zero = state.emit_i256_constant(0, &block);
                    block.append_operation(
                        melior::ir::operation::OperationBuilder::new(solx_mlir::ops::RETURN, location)
                            .add_operands(&[zero])
                            .build()
                            .expect("llvm.return operation is well-formed"),
                    );
                } else {
                    block.append_operation(llvm::unreachable(location));
                }
            }
            maybe_block = block.next_in_region();
        }
    }

    /// Emits a default `llvm.return` for a block (returns 0 for non-void).
    fn emit_default_return(&self, has_returns: bool, block: &melior::ir::BlockRef<'context, '_>) {
        let location = self.state.location();
        if has_returns {
            let zero = self.state.emit_i256_constant(0, block);
            block.append_operation(
                melior::ir::operation::OperationBuilder::new(solx_mlir::ops::RETURN, location)
                    .add_operands(&[zero])
                    .build()
                    .expect("llvm.return operation is well-formed"),
            );
        } else {
            block.append_operation(
                melior::ir::operation::OperationBuilder::new(solx_mlir::ops::RETURN, location)
                    .build()
                    .expect("llvm.return operation is well-formed"),
            );
        }
    }
}
