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
use std::sync::Once;

use melior::dialect::DialectRegistry;
use melior::dialect::ods::llvm::ModuleFlagsOperation;
use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Location;
use melior::ir::Module;
use melior::ir::attribute::ArrayAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::operation::OperationMutLike;
use melior::ir::operation::OperationPrintingFlags;
use melior::pass::PassManager;
use solx_utils::Profiler;

use crate::Block;
use crate::DebugInfoCompileUnit;
use crate::FunctionOrigin;
use crate::Type;

use self::pass_timing::PassTiming;

/// Accumulated MLIR state threaded through the AST visitors.
pub struct Context<'context> {
    /// The MLIR context.
    pub melior: &'context melior::Context,
    /// The MLIR module being built.
    pub module: Module<'context>,
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

    /// Whether LLVM is built with threads. Without them its locks do nothing, so only one thread
    /// at a time may use LLVM and MLIR.
    pub fn is_multithreaded() -> bool {
        inkwell::support::is_multithreaded()
    }

    /// Creates a new MLIR state with an empty module at `location`, with a compile unit of
    /// `file_name` fused onto it with `emit_debug_info`. `location` is
    /// the object's definition, which is where the Sol-to-Yul lowering puts the functions it
    /// generates, or the unknown location without debug info. The location cursor starts at
    /// `location`.
    pub fn new(
        melior: &'context melior::Context,
        evm_version: solx_utils::EVMVersion,
        revert_strings: solx_utils::RevertStrings,
        location: Location<'context>,
        emit_debug_info: bool,
        file_name: &str,
    ) -> Self {
        for dialect in Self::EMITTED_DIALECTS {
            melior.get_or_load_dialect(dialect);
        }

        let debug_info_compile_unit =
            emit_debug_info.then(|| DebugInfoCompileUnit::new(melior, file_name));
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
        if emit_debug_info {
            Self::declare_dwarf_version(melior, &module);
        }

        Self {
            melior,
            module,
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
                crate::ffi::mlirCreateSolModifierInliningPass(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateTransformsCanonicalizer(),
            ));
            pass_manager.add_pass(melior::pass::Pass::from_raw(
                crate::ffi::mlirCreateConversionConvertSolToYulPass(),
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

    /// Runs the pass pipeline on a code segment's module and translates it to LLVM bitcode,
    /// returned with `dependencies`, the objects the code may embed, whose identifier must match
    /// the segment's object.
    ///
    /// With `capture_mlir`, the Sol text and the LLVM dialect text are printed too, with locations
    /// ([`Self::printing_flags`]) when debug info is requested for the segment.
    ///
    /// `pass_timing` records the time of every pass in `profiler`, under
    /// `Compiler_RunSolPasses:<code_identifier>/`.
    ///
    /// # Errors
    ///
    /// Returns an error if the pass pipeline fails, or the module cannot be printed or translated.
    pub fn finalize_module(
        self,
        dependencies: solx_utils::Dependencies,
        capture_mlir: bool,
        pass_timing: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<crate::output::SegmentOutput> {
        let code_identifier = dependencies.identifier.as_str();
        let mut module = self.module;

        let sol_source = capture_mlir
            .then(|| {
                module
                    .as_operation()
                    .to_string_with_flags(Self::printing_flags(
                        self.debug_info_compile_unit.is_some(),
                    ))
            })
            .transpose()
            .map_err(|error| anyhow::anyhow!("Sol dialect module printing: {error}"))?;

        let sol_passes_label = format!("Compiler_RunSolPasses:{code_identifier}");
        let run_sol_passes = profiler.start_pipeline_element(sol_passes_label.as_str());
        let pass_timings = Self::run_sol_passes(self.melior, &mut module, pass_timing)?;
        run_sol_passes.borrow_mut().finish();
        Self::record_pass_timings(profiler, sol_passes_label.as_str(), pass_timings);

        let source = capture_mlir
            .then(|| {
                module
                    .as_operation()
                    .to_string_with_flags(Self::printing_flags(
                        self.debug_info_compile_unit.is_some(),
                    ))
            })
            .transpose()
            .map_err(|error| anyhow::anyhow!("LLVM dialect module printing: {error}"))?;

        let run_translation = profiler
            .start_pipeline_element(format!("Compiler_MLIRToLLVMIR:{code_identifier}").as_str());
        let bitcode = Self::translate_to_bitcode(&module)?;
        run_translation.borrow_mut().finish();

        Ok(crate::output::SegmentOutput {
            sol_source,
            source,
            bitcode,
            dependencies,
        })
    }

    /// Translates an LLVM dialect module to LLVM IR and returns its bitcode, which codegen parses
    /// into the LLVM context it compiles in.
    ///
    /// # Errors
    ///
    /// Returns an error if the module cannot be translated to LLVM IR.
    fn translate_to_bitcode(module: &Module) -> anyhow::Result<Vec<u8>> {
        let llvm = inkwell::context::Context::create();
        let raw_module = unsafe {
            mlir_sys::mlirTranslateModuleToLLVMIR(
                module.as_operation().to_raw(),
                llvm.raw() as *mut _,
            )
        };
        if raw_module.is_null() {
            anyhow::bail!("mlirTranslateModuleToLLVMIR returned null");
        }
        let llvm_module = unsafe { inkwell::module::Module::new(raw_module as *mut _) };
        Ok(llvm_module.write_bitcode_to_memory().as_slice().to_vec())
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

    /// The flags a module text is printed with. `print_locations` prints locations, which the
    /// default flags drop, in the form that re-parses. Each distinct location is written once as a
    /// `#locN` alias, except on block arguments, which the printer allows no alias.
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
    /// MLIR's translation sets `Debug Info Version` but no DWARF version. Each segment's module
    /// becomes its own LLVM module, so each segment that carries debug info declares its own.
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
}
