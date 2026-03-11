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

use slang_solidity::backend::ir::ir2_flat_contracts::FunctionDefinition;

use solx_mlir::Environment;
use solx_mlir::ops;

use crate::codegen::MlirContext;
use crate::codegen::expression::ExpressionEmitter;
use crate::codegen::statement::StatementEmitter;
use crate::codegen::types::TypeMapper;

/// Lowers a Solidity function definition to an MLIR `llvm.func` operation.
pub struct FunctionEmitter<'a, 'c> {
    /// The shared MLIR context.
    state: &'a MlirContext<'c>,
}

impl<'a, 'c> FunctionEmitter<'a, 'c> {
    /// Creates a new function emitter.
    pub fn new(state: &'a MlirContext<'c>) -> Self {
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
    pub fn emit(&self, func: &FunctionDefinition) -> anyhow::Result<String> {
        let name = func
            .name
            .as_ref()
            .map(|terminal| terminal.text.as_str())
            .unwrap_or("unnamed");

        let param_types: Vec<String> = func
            .parameters
            .iter()
            .map(|p| TypeMapper::canonical_type(&p.type_name))
            .collect();
        let mlir_name = format!("solx.fn.{name}({})", param_types.join(","));

        let context = self.state.context();
        let location = self.state.location();
        let i256 = self.state.i256();

        let has_returns = func
            .returns
            .as_ref()
            .is_some_and(|returns| !returns.is_empty());

        let mlir_param_types: Vec<melior::ir::Type<'c>> =
            func.parameters.iter().map(|_| i256).collect();

        let func_type = llvm::r#type::function(
            if has_returns {
                i256
            } else {
                llvm::r#type::void(context)
            },
            &mlir_param_types,
            false,
        );

        let region = Region::new();
        let block_args: Vec<(melior::ir::Type<'c>, melior::ir::Location<'c>)> =
            mlir_param_types.iter().map(|t| (*t, location)).collect();
        let entry_block = region.append_block(Block::new(&block_args));

        let mut env = Environment::new();

        // Create allocas for parameters and bind to environment.
        for (i, param) in func.parameters.iter().enumerate() {
            let param_name = param
                .name
                .as_ref()
                .map(|t| t.text.as_str())
                .unwrap_or("_");
            let param_value: melior::ir::Value<'c, '_> = entry_block.argument(i)?.into();

            let expr_emitter = ExpressionEmitter::new(self.state, &env, &region);
            let ptr = expr_emitter.emit_alloca(&entry_block);
            expr_emitter.emit_store(param_value, ptr, &entry_block)?;

            if TypeMapper::is_signed(&param.type_name) {
                env.mark_signed(param_name);
            }
            env.define_variable(param_name.to_owned(), ptr);
        }

        if let Some(ref body) = func.body {
            let mut current_block = entry_block;
            for stmt in &body.statements {
                let mut emitter = StatementEmitter::new(self.state, &mut env, &region);
                match emitter.emit(stmt, current_block)? {
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

        let func_op = llvm::func(
            context,
            StringAttribute::new(context, &mlir_name),
            TypeAttribute::new(func_type),
            region,
            &[],
            location,
        );

        self.state.body().append_operation(func_op);

        Ok(mlir_name)
    }

    /// Adds `llvm.unreachable` to any block that lacks a terminator.
    ///
    /// Dead blocks can appear from control flow constructs (e.g. `for` loop
    /// exit/iter blocks when the body always returns). MLIR requires every
    /// block to end with a terminator.
    fn terminate_empty_blocks(region: &Region<'c>, has_returns: bool, state: &MlirContext<'c>) {
        let location = state.location();
        let mut maybe_block = region.first_block();
        while let Some(block) = maybe_block {
            let needs_terminator = block.terminator().is_none();
            if needs_terminator {
                if has_returns {
                    let zero = state.emit_i256_constant(0, &block);
                    block.append_operation(
                        melior::ir::operation::OperationBuilder::new(ops::RETURN, location)
                            .add_operands(&[zero])
                            .build()
                            .expect("valid llvm.return"),
                    );
                } else {
                    block.append_operation(llvm::unreachable(location));
                }
            }
            maybe_block = block.next_in_region();
        }
    }

    /// Emits a default `llvm.return` for a block (returns 0 for non-void).
    fn emit_default_return(&self, has_returns: bool, block: &melior::ir::BlockRef<'c, '_>) {
        let location = self.state.location();
        if has_returns {
            let zero = self.state.emit_i256_constant(0, block);
            block.append_operation(
                melior::ir::operation::OperationBuilder::new(ops::RETURN, location)
                    .add_operands(&[zero])
                    .build()
                    .expect("valid llvm.return"),
            );
        } else {
            block.append_operation(
                melior::ir::operation::OperationBuilder::new(ops::RETURN, location)
                    .build()
                    .expect("valid llvm.return"),
            );
        }
    }
}
