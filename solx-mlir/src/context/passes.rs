//!
//! Sol dialect pass pipeline and module finalization.
//!

use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::Module as MlirModule;
use melior::ir::operation::OperationLike;
use melior::pass::PassManager;

use crate::context::Context;

impl Context {
    /// Run the Sol-to-LLVM conversion pass pipeline on a module in-place.
    ///
    /// The pass pipeline is:
    /// 1. `convert-sol-to-std` — Sol + Yul → func/arith/scf/cf/LLVM
    /// 2. `convert-func-to-llvm`
    /// 3. `convert-scf-to-cf`
    /// 4. `convert-cf-to-llvm`
    /// 5. `convert-arith-to-llvm`
    /// 6. `reconcile-unrealized-casts`
    ///
    /// Modifier lowering and LICM are skipped — they operate on `sol.modifier`
    /// and `sol.while`/`sol.for` ops which are not yet emitted.
    ///
    /// # Errors
    ///
    /// Returns an error if any pass in the pipeline fails.
    pub fn run_sol_passes(
        context: &melior::Context,
        module: &mut MlirModule,
    ) -> Result<(), String> {
        let pass_manager = PassManager::new(context);
        pass_manager.enable_verifier(true);

        // SAFETY: Each `mlirCreate*Pass` returns a freshly allocated pass
        // object. `Pass::from_raw` takes ownership. The pass manager runs
        // them sequentially on the module. No aliasing or use-after-free.
        unsafe {
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToStandardPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertFuncToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionSCFToControlFlowPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertControlFlowToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionArithToLLVMConversionPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionReconcileUnrealizedCastsPass(),
            ));
        }

        pass_manager
            .run(module)
            .map_err(|e| format!("Sol pass pipeline failed: {e}"))
    }

    /// Consumes the builder, runs the Sol-to-LLVM pass pipeline, and returns
    /// the resulting LLVM dialect MLIR as text.
    ///
    /// The Sol conversion pass produces a nested module structure:
    /// ```text
    /// module @Contract { deploy __entry + module @Contract_deployed { runtime __entry } }
    /// ```
    /// `solx-core` provides its own deploy wrapper, so this method extracts
    /// only the inner `_deployed` module and renames it to
    /// `runtime_code_identifier` so the LLVM module identifier matches what
    /// `minimal_deploy_code` references.
    ///
    /// # Errors
    ///
    /// Returns an error if re-parsing fails, the pass pipeline fails, or
    /// the deployed module is not found.
    pub fn finalize_module(
        &self,
        builder: crate::MlirContext<'_>,
        runtime_code_identifier: &str,
    ) -> anyhow::Result<String> {
        let module = builder.into_module();

        // Re-parse to promote OperationBuilder dict attributes to properties.
        let sol_text = module.as_operation().to_string();
        let mut parsed_module = MlirModule::parse(self.mlir(), &sol_text).ok_or_else(|| {
            anyhow::anyhow!("failed to re-parse generated Sol dialect MLIR:\n{sol_text}")
        })?;

        // Lower Sol → LLVM dialect.
        Self::run_sol_passes(self.mlir(), &mut parsed_module)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Walk the outer module's body to find the inner `_deployed` module
        // and extract it as the runtime code. Rename it so the LLVM module
        // identifier matches what the deploy stub references.
        let body = parsed_module.body();
        let mut deployed_operation = None;
        let mut operation = body.first_operation();
        while let Some(current) = operation {
            if current.name().as_string_ref().as_str().unwrap_or("") == "builtin.module"
                && let Ok(symbol) = current.attribute("sym_name")
            {
                let symbol_string = symbol.to_string();
                if symbol_string.contains(runtime_code_identifier) {
                    deployed_operation = Some(current);
                    break;
                }
            }
            operation = current.next_in_block();
        }

        let runtime_op = deployed_operation
            .ok_or_else(|| anyhow::anyhow!("no _deployed module in Sol pass output"))?;

        // SAFETY: Setting `sym_name` on a valid MLIR operation. The
        // operation, string ref, and attribute are all valid MLIR objects.
        unsafe {
            mlir_sys::mlirOperationSetAttributeByName(
                runtime_op.to_raw(),
                mlir_sys::mlirStringRefCreateFromCString(c"sym_name".as_ptr()),
                melior::ir::attribute::StringAttribute::new(self.mlir(), runtime_code_identifier)
                    .to_raw(),
            );
        }

        // Serialize only the deployed module (now with the right name).
        Ok(runtime_op.to_string())
    }
}
