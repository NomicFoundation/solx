//!
//! MLIR compilation context for EVM code generation.
//!

pub mod contract;
pub mod debug_info;
pub mod environment;
pub mod function;
pub mod pass_timing;
pub mod yul_function;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::CString;
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::dialect::ods::llvm::ModuleFlagsOperation;
use melior::ir::Attribute;
use melior::ir::AttributeLike;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::Operation;
use melior::ir::attribute::ArrayAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationMutLike;
use melior::ir::operation::OperationPrintingFlags;
use melior::ir::operation::OperationRef;
use melior::ir::operation::WalkOrder;
use melior::ir::operation::WalkResult;
use melior::pass::PassManager;
use solx_utils::Profiler;

use crate::Block;
use crate::DebugInfoCompileUnit;
use crate::DebugInfoRequest;
use crate::FunctionOrigin;
use crate::Type;
use crate::llvm_module::RawLlvmModule;

use self::pass_timing::PassTiming;

/// Accumulated MLIR state threaded through the AST visitors.
pub struct Context<'context> {
    /// The MLIR context.
    pub melior: &'context melior::Context,
    /// The MLIR module being built.
    pub module: Module<'context>,
    /// Which of the module's code segments debug info is requested for.
    pub debug_info_request: DebugInfoRequest,
    /// The debug-info compile unit every subprogram in this module points at. Absent without debug
    /// info, where no function gets a subprogram.
    pub debug_info_compile_unit: Option<DebugInfoCompileUnit<'context>>,
    /// The location cursor: the location the next op carries, that of the innermost node being
    /// lowered. Frontends set it around each node and restore the enclosing value.
    pub current_location: Location<'context>,
    /// The MLIR type of the contract currently being emitted, used to type
    /// `this` expressions. Frontends set this before emitting function bodies.
    pub current_contract_type: Option<Type<'context>>,
    /// The block value producers and effects append to during function-body emission. The
    /// control-flow emitters move it onto region entry blocks; it is absent between functions.
    pub current_block: Option<Block<'context>>,
}

impl<'context> Context<'context> {
    /// The dialects emission builds in, which op, type and attribute construction does not load.
    const EMITTED_DIALECTS: [&'static str; 2] = ["sol", "yul"];
    /// The op a code segment's module is.
    const BUILTIN_MODULE: &'static str = "builtin.module";
    /// The DWARF version a module with debug info declares, where LLVM would default to 4.
    const DWARF_VERSION: u32 = 5;
    /// The data layout the LLVM translation reads off the module.
    const DATA_LAYOUT: &'static str = "llvm.data_layout";
    /// The target triple the LLVM translation reads off the module.
    const TARGET_TRIPLE: &'static str = "llvm.target_triple";
    /// The EVM version the `convert-sol-to-yul` pass reads off the module.
    const EVM_VERSION: &'static str = "sol.evm_version";
    /// The revert-string policy the `convert-sol-to-yul` pass reads off the module.
    const REVERT_STRINGS: &'static str = "sol.revert_strings";
    /// The attribute a `builtin.module` carries its own identifier in.
    const MODULE_SYMBOL: &'static str = "sym_name";

    /// The op the object-naming intrinsics reach as, after the Yul-to-standard pass.
    const LLVM_INTRINSIC_CALL: &'static str = "llvm.intrcall";
    /// The intrinsics naming an object: the segment's own code, the runtime child its deploy
    /// segment returns, and the contract a creation copies its bytecode from.
    const OBJECT_INTRINSICS: [&'static str; 2] = ["evm.dataoffset", "evm.datasize"];
    /// The attribute an intrinsic call carries its own name in.
    const INTRINSIC_NAME: &'static str = "name";
    /// The attribute an intrinsic call carries its string operands in.
    const INTRINSIC_METADATA: &'static str = "metadata";

    /// Creates a single-threaded MLIR context.
    ///
    /// `register_all_llvm_translations` MUST be called before any
    /// MLIR-to-LLVM translation. Without it, `mlirTranslateModuleToLLVMIR`
    /// returns null. This function enforces that invariant.
    pub fn create_melior_context() -> melior::Context {
        let registry = DialectRegistry::new();
        melior::utility::register_all_dialects(&registry);

        unsafe {
            crate::ffi::mlirDialectHandleInsertDialect(
                crate::ffi::mlirGetDialectHandle__sol__(),
                registry.to_raw(),
            );
            crate::ffi::mlirDialectHandleInsertDialect(
                crate::ffi::mlirGetDialectHandle__yul__(),
                registry.to_raw(),
            );
        }

        let melior = melior::Context::new();
        melior.enable_multi_threading(false);
        melior.append_dialect_registry(&registry);
        melior::utility::register_all_llvm_translations(&melior);

        static REGISTER_PASSES: Once = Once::new();
        REGISTER_PASSES.call_once(|| unsafe {
            crate::ffi::mlirRegisterSolPasses();
        });

        melior
    }

    /// Creates a new MLIR state with an empty module at `location`, with a compile unit of
    /// `file_name` fused onto it when `debug_info_request` asks for either segment. `location` is
    /// the object's definition, which is where the Sol-to-Yul lowering puts the functions it
    /// generates, or the unknown location without debug info. The location cursor starts at
    /// `location`.
    pub fn new(
        melior: &'context melior::Context,
        evm_version: solx_utils::EVMVersion,
        revert_strings: solx_utils::RevertStrings,
        location: Location<'context>,
        debug_info_request: DebugInfoRequest,
        file_name: &str,
    ) -> Self {
        for dialect in Self::EMITTED_DIALECTS {
            melior.get_or_load_dialect(dialect);
        }

        let debug_info_compile_unit = debug_info_request
            .any()
            .then(|| DebugInfoCompileUnit::new(melior, file_name));
        let module_location = match debug_info_compile_unit {
            Some(debug_info_compile_unit) => {
                debug_info_compile_unit.fuse_compile_unit(melior, location)
            }
            None => location,
        };
        let mut module = Module::new(module_location);

        let evm_version_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateEvmVersionAttr(
                melior.to_raw(),
                evm_version.into_sol_dialect_identifier(),
            ))
        };
        let revert_strings_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateRevertStringsAttr(
                melior.to_raw(),
                revert_strings as u32,
            ))
        };
        let target = solx_utils::Target::EVM;
        let mut operation = module.as_operation_mut();
        operation.set_attribute(Self::EVM_VERSION, evm_version_attribute);
        operation.set_attribute(Self::REVERT_STRINGS, revert_strings_attribute);
        operation.set_attribute(
            Self::DATA_LAYOUT,
            StringAttribute::new(melior, target.data_layout()).into(),
        );
        operation.set_attribute(
            Self::TARGET_TRIPLE,
            StringAttribute::new(melior, target.triple()).into(),
        );

        Self {
            melior,
            module,
            debug_info_request,
            debug_info_compile_unit,
            current_location: location,
            current_contract_type: None,
            current_block: None,
        }
    }

    /// The block the insertion cursor points at, which the function-body emitters position before
    /// any emission.
    pub fn current_block(&self) -> Block<'context> {
        self.current_block
            .expect("the function body positions the insertion cursor")
    }

    /// Run the Sol-to-LLVM conversion pass pipeline on a module in-place.
    ///
    /// The first `symbol-dce` removes the functions unreachable in the contract before anything
    /// walks them. Splitting the contract into a creation and a runtime object can leave a
    /// function unreachable in the runtime object; the second takes care of that.
    ///
    /// # Errors
    ///
    /// Returns an error if any pass in the pipeline fails or the resulting module fails
    /// verification.
    pub fn run_sol_passes(
        melior: &melior::Context,
        module: &mut Module,
        pass_timing: bool,
    ) -> anyhow::Result<Vec<PassTiming>> {
        let mut pass_timings = Vec::new();
        let pass_manager = PassManager::new(melior);
        pass_manager.enable_verifier(cfg!(debug_assertions));
        if pass_timing {
            unsafe {
                crate::ffi::solxPassManagerEnableTiming(
                    pass_manager.to_raw(),
                    PassTiming::push,
                    (&raw mut pass_timings).cast(),
                )
            };
        }

        unsafe {
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsSymbolDCE(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateSolModifierInliningPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToYulPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsSymbolDCE(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertYulToStandardPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionSCFToControlFlowPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertFuncToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionArithToLLVMConversionPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertControlFlowToLLVMPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionReconcileUnrealizedCastsPass(),
            ));
        }

        pass_manager
            .run(module)
            .map_err(|error| anyhow::anyhow!("Sol pass pipeline failed: {error}"))?;
        // The timing report is written into `pass_timings` when the pass manager is destroyed, so
        // the destruction has to happen before `pass_timings` moves out.
        drop(pass_manager);
        if !cfg!(debug_assertions) && !module.as_operation().verify() {
            anyhow::bail!("Sol pass pipeline produced an invalid module");
        }
        Ok(pass_timings)
    }

    /// Returns the deploy and runtime modules as separate LLVM dialect strings.
    ///
    /// The Sol conversion pass produces a nested module:
    /// ```text
    /// module @Contract { deploy __entry + module @Contract_deployed { runtime __entry } }
    /// ```
    /// Each is translated to its own LLVM IR module and emits its own bytecode segment. The outer
    /// carries the deploy entry that runs the constructor and returns the runtime bytecode.
    ///
    /// Each segment's text is printed with locations ([`Self::printing_flags`]) when debug info
    /// is requested for it, and the Sol text when it is for either.
    ///
    /// `pass_timing` records the time of every pass in `profiler`, under
    /// `slang_RunSolPasses:<code_identifier>/`.
    ///
    /// # Errors
    ///
    /// Returns an error if the pass pipeline fails, the runtime module is
    /// not found, or a module cannot be printed.
    pub fn finalize_module(
        self,
        code_identifier: &str,
        capture_sol: bool,
        pass_timing: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<crate::output::MlirOutput> {
        let mut module = self.module;

        let sol_source = capture_sol
            .then(|| {
                module
                    .as_operation()
                    .to_string_with_flags(Self::printing_flags(self.debug_info_request.any()))
            })
            .transpose()
            .map_err(|error| anyhow::anyhow!("Sol dialect module printing: {error}"))?;

        let sol_passes_label = format!("slang_RunSolPasses:{code_identifier}");
        let run_sol_passes = profiler.start_pipeline_element(sol_passes_label.as_str());
        let pass_timings = Self::run_sol_passes(self.melior, &mut module, pass_timing)?;
        run_sol_passes.borrow_mut().finish();
        Self::record_pass_timings(profiler, sol_passes_label.as_str(), pass_timings);

        if self.debug_info_request.deploy {
            Self::declare_dwarf_version(self.melior, &module);
        }

        let run_object_extraction = profiler
            .start_pipeline_element(format!("slang_ExtractMLIRObjects:{code_identifier}").as_str());
        let runtime_code_identifier = format!(
            "{code_identifier}{}",
            solx_utils::Dependencies::DEPLOYED_OBJECT_SUFFIX
        );
        let (runtime_llvm, runtime_dependencies) = Self::take_nested_module(
            self.melior,
            &mut module,
            runtime_code_identifier.as_str(),
            self.debug_info_request.runtime,
        )?;
        let deploy_dependencies = Self::object_dependencies(
            &module.as_operation(),
            code_identifier,
            Some(runtime_code_identifier),
        );
        let deploy_llvm = module
            .as_operation()
            .to_string_with_flags(Self::printing_flags(self.debug_info_request.deploy))
            .map_err(|error| anyhow::anyhow!("deploy module printing: {error}"))?;
        run_object_extraction.borrow_mut().finish();

        Ok(crate::output::MlirOutput {
            sol_source,
            deploy_source: deploy_llvm,
            deploy_dependencies,
            runtime_source: runtime_llvm,
            runtime_dependencies,
        })
    }

    /// Parses MLIR source text (LLVM dialect) into a verified module.
    ///
    /// # Errors
    ///
    /// Returns an error if the source cannot be parsed or fails verification.
    pub fn parse_source<'melior>(
        melior: &'melior melior::Context,
        source: &str,
    ) -> anyhow::Result<Module<'melior>> {
        let module = Module::parse(melior, source)
            .ok_or_else(|| anyhow::anyhow!("failed to parse MLIR source text"))?;

        if !module.as_operation().verify() {
            anyhow::bail!("MLIR module verification failed");
        }

        Ok(module)
    }

    /// Translates a parsed LLVM-dialect module to raw LLVM pointers.
    ///
    /// The module is consumed because lowering `llvm.setimmutable` erases the operations it reads,
    /// and the translation copies everything it needs.
    ///
    /// # Errors
    ///
    /// Returns an error if the module cannot be translated to LLVM IR.
    pub fn translate_module_to_llvm(
        module: Module,
        immutables: &BTreeMap<String, BTreeSet<u64>>,
    ) -> anyhow::Result<RawLlvmModule> {
        let ids: Vec<CString> = immutables
            .keys()
            .map(|id| CString::new(id.as_str()).expect("an immutable id carries no NUL byte"))
            .collect();
        let (id_pointers, offsets): (Vec<*const std::ffi::c_char>, Vec<u64>) = ids
            .iter()
            .zip(immutables.values())
            .flat_map(|(id, offsets)| offsets.iter().map(|offset| (id.as_ptr(), *offset)))
            .unzip();

        unsafe {
            crate::ffi::mlirEvmLowerSetImmutables(
                module.to_raw(),
                id_pointers.as_ptr(),
                offsets.as_ptr(),
                offsets.len() as u64,
            );

            let raw_operation = module.as_operation().to_raw();
            let llvm_context = inkwell::llvm_sys::core::LLVMContextCreate();

            let llvm_module =
                mlir_sys::mlirTranslateModuleToLLVMIR(raw_operation, llvm_context as *mut _);

            if llvm_module.is_null() {
                inkwell::llvm_sys::core::LLVMContextDispose(llvm_context);
                anyhow::bail!("mlirTranslateModuleToLLVMIR returned null");
            }

            Ok(RawLlvmModule {
                context: llvm_context,
                module: llvm_module as *mut _,
            })
        }
    }

    /// The location a `sol.func` or `yul.func` carries: the location cursor with a subprogram of
    /// its own fused onto it, artificial for a function the frontend synthesizes.
    ///
    /// The translation keeps a function's inner locations only when its own location carries a
    /// subprogram, which belongs there and nowhere else
    /// ([`DebugInfoCompileUnit::fuse_subprogram`]).
    fn function_location(&self, name: &str, origin: FunctionOrigin) -> Location<'context> {
        match self.debug_info_compile_unit {
            Some(debug_info_compile_unit) => {
                debug_info_compile_unit.fuse_subprogram(name, origin, self.current_location)
            }
            None => self.current_location,
        }
    }

    /// The flags a module text is printed with. `print_locations` prints locations in the
    /// non-pretty form, the one that re-parses, so they survive the round-trip to the worker
    /// processes; the default flags drop them. Each distinct location is written once as a `#locN`
    /// alias, except on block arguments, which the printer allows no alias.
    fn printing_flags(print_locations: bool) -> OperationPrintingFlags {
        let flags = OperationPrintingFlags::new();
        if print_locations {
            flags.enable_debug_info(true, false)
        } else {
            flags
        }
    }

    /// Appends to `module` the `llvm.module_flags` declaring [`Self::DWARF_VERSION`], at the
    /// module's location.
    ///
    /// MLIR's translation sets `Debug Info Version` but no DWARF version. Each module a worker
    /// translates becomes its own LLVM module, so each segment that carries debug info declares
    /// its own.
    fn declare_dwarf_version(melior: &'context melior::Context, module: &Module<'context>) {
        let flag = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateDwarfVersionFlagAttr(
                melior.to_raw(),
                Self::DWARF_VERSION,
            ))
        };
        module.body().append_operation(
            ModuleFlagsOperation::builder(melior, module.as_operation().location())
                .flags(ArrayAttribute::new(melior, &[flag]))
                .build()
                .into(),
        );
    }

    /// Records each pass timing under `label`, the analyses a pass ran nested under its entry and
    /// a pass's n-th run as `<pass> #<n>`.
    fn record_pass_timings(profiler: &mut Profiler, label: &str, pass_timings: Vec<PassTiming>) {
        let mut runs = BTreeMap::<String, usize>::new();
        let mut path = Vec::new();
        for pass_timing in pass_timings {
            path.truncate(pass_timing.depth as usize);
            if pass_timing.depth > 0 {
                path.push(pass_timing.name);
            } else {
                let run = runs.entry(pass_timing.name.clone()).or_default();
                *run += 1;
                if *run == 1 {
                    path.push(pass_timing.name);
                } else {
                    path.push(format!("{} #{run}", pass_timing.name));
                }
            }
            profiler.record_pipeline_element(
                format!("{label}/{}", path.join("/")).as_str(),
                pass_timing.duration,
            );
        }
    }

    /// The walk runs before the module leaves the tree, so each segment's dependencies come from
    /// its own code.
    ///
    /// With `print_locations`, the module declares the DWARF version and is detached before it is
    /// printed: the printer emits an alias table only for a top-level op, so a detached module
    /// writes each repeated location once and re-parses on its own. Without, it is printed in
    /// place, which spares the printer the scan that builds the table.
    fn take_nested_module(
        melior: &'context melior::Context,
        module: &mut Module<'context>,
        target: &str,
        print_locations: bool,
    ) -> anyhow::Result<(String, solx_utils::Dependencies)> {
        let body = module.body();
        std::iter::successors(body.first_operation_mut(), |operation| {
            operation.next_in_block_mut()
        })
        .find_map(|mut operation| {
            if operation.name().as_string_ref().as_str() != Ok(Self::BUILTIN_MODULE) {
                return None;
            }
            let symbol: StringAttribute = operation
                .attribute(Self::MODULE_SYMBOL)
                .ok()?
                .try_into()
                .ok()?;
            if symbol.value() != target {
                return None;
            }

            let dependencies = Self::object_dependencies(&operation, target, None);

            let text = if print_locations {
                operation.remove_from_parent();
                let runtime =
                    Module::from_operation(unsafe { Operation::from_raw(operation.to_raw()) })
                        .expect("a `builtin.module` op is a module");
                Self::declare_dwarf_version(melior, &runtime);
                runtime
                    .as_operation()
                    .to_string_with_flags(Self::printing_flags(true))
                    .map_err(|error| anyhow::anyhow!("runtime module printing: {error}"))
            } else {
                let text = operation.to_string();
                operation.remove_from_parent();
                drop(unsafe { Operation::from_raw(operation.to_raw()) });
                Ok(text)
            };

            Some(text.map(|text| (text, dependencies)))
        })
        .ok_or_else(|| anyhow::anyhow!("no module with sym_name `{target}` in Sol pass output"))?
    }

    /// The objects `operation`'s code references, read off the intrinsics naming them.
    fn object_dependencies<'c: 'a, 'a>(
        operation: &impl OperationLike<'c, 'a>,
        identifier: &str,
        runtime: Option<String>,
    ) -> solx_utils::Dependencies {
        let mut dependencies = solx_utils::Dependencies::new(identifier, runtime);
        operation.walk(WalkOrder::PreOrder, |operation| {
            if let Some(object) = Self::referenced_object(operation) {
                dependencies.push(object);
            }
            WalkResult::Advance
        });
        dependencies
    }

    /// The object an `evm.dataoffset` / `evm.datasize` intrinsic call names.
    fn referenced_object(operation: OperationRef<'_, '_>) -> Option<String> {
        if operation.name().as_string_ref().as_str().ok()? != Self::LLVM_INTRINSIC_CALL {
            return None;
        }
        let name: StringAttribute = operation
            .attribute(Self::INTRINSIC_NAME)
            .ok()?
            .try_into()
            .ok()?;
        if !Self::OBJECT_INTRINSICS.contains(&name.value()) {
            return None;
        }
        let metadata = operation
            .attribute(Self::INTRINSIC_METADATA)
            .expect("an object intrinsic names its object in `metadata`");
        let object: StringAttribute =
            unsafe { Attribute::from_raw(mlir_sys::mlirArrayAttrGetElement(metadata.to_raw(), 0)) }
                .try_into()
                .expect("`metadata` is a one-element string array");
        Some(object.value().to_owned())
    }
}
