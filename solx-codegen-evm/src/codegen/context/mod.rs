//!
//! The LLVM module build context.
//!

use solx_utils::Profiler;

use crate::attribute::Attribute;
use crate::codegen::build::Build as EVMBuild;
use crate::debug_config::OutputConfig;
use crate::optimizer::Optimizer;
use crate::optimizer::settings::Settings as OptimizerSettings;
use crate::target_machine::TargetMachine;

///
/// The LLVM module build context.
///
/// Owns the module from its translation to the emitted bytecode.
///
pub struct Context<'ctx> {
    /// The inner LLVM context.
    llvm: &'ctx inkwell::context::Context,
    /// The optimization tools.
    optimizer: Optimizer,
    /// The current module.
    module: inkwell::module::Module<'ctx>,
    /// The extra LLVM options.
    llvm_options: Vec<String>,
    /// The current contract code type, which can be deploy or runtime.
    code_segment: solx_utils::CodeSegment,
    /// The output configuration telling whether to dump the needed IRs.
    output_config: Option<OutputConfig>,
    /// Whether to capture LLVM IR for output.
    capture_llvm_ir: bool,
}

impl<'ctx> Context<'ctx> {
    ///
    /// Initializes a new LLVM context.
    ///
    pub fn new(
        llvm: &'ctx inkwell::context::Context,
        module: inkwell::module::Module<'ctx>,
        llvm_options: Vec<String>,
        code_segment: solx_utils::CodeSegment,
        optimizer: Optimizer,
        output_config: Option<OutputConfig>,
    ) -> Self {
        Self {
            llvm,
            optimizer,
            module,
            llvm_options,
            code_segment,
            output_config,
            capture_llvm_ir: false,
        }
    }

    ///
    /// Builds the LLVM IR module, returning the build artifacts.
    ///
    pub fn build(
        &mut self,
        output_assembly: bool,
        output_bytecode: bool,
        is_size_fallback: bool,
        profiler: &mut Profiler,
    ) -> anyhow::Result<EVMBuild> {
        let contract_path = self.module.get_name().to_str().expect("Always valid");
        let optimizer_mode = self.optimizer.settings().to_string();
        let spill_area_size = self.optimizer.settings().spill_area_size();

        let run_init_verify = profiler.start_evm_translation_unit(
            contract_path,
            Some(self.code_segment),
            "InitVerify",
            optimizer_mode.as_str(),
            spill_area_size,
        );
        let target_machine = TargetMachine::new(
            self.optimizer.settings(),
            self.llvm_options.as_slice(),
            spill_area_size.map(|size| (crate::r#const::SOLC_USER_MEMORY_OFFSET, size)),
        )?;
        target_machine.set_target_data(&self.module);
        target_machine.set_asm_verbosity(true);

        if let Some(output_config) = self.output_config.as_ref() {
            output_config.dump_llvm_ir_unoptimized(
                contract_path,
                &self.module,
                is_size_fallback,
                spill_area_size,
            )?;
        }
        // Capture unoptimized LLVM IR for output if requested and not writing to files
        let captured_llvm_ir_unoptimized = if self.capture_llvm_ir && self.output_config.is_none() {
            Some(self.module.print_to_string().to_string())
        } else {
            None
        };
        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "{} code unoptimized LLVM IR verification: {error}",
                self.code_segment,
            )
        })?;
        run_init_verify.borrow_mut().finish();

        let needs_size_fallback = self.optimizer.settings() == &OptimizerSettings::cycles()
            && self.optimizer.settings().is_fallback_to_size_enabled();
        let module_size_fallback = needs_size_fallback.then(|| self.module.clone());

        let run_optimize_verify = profiler.start_evm_translation_unit(
            contract_path,
            Some(self.code_segment),
            "OptimizeVerify",
            optimizer_mode.as_str(),
            spill_area_size,
        );
        self.optimizer
            .run(&target_machine, &self.module)
            .map_err(|error| anyhow::anyhow!("{} code optimizing: {error}", self.code_segment))?;
        if let Some(output_config) = self.output_config.as_ref() {
            output_config.dump_llvm_ir_optimized(
                contract_path,
                &self.module,
                is_size_fallback,
                spill_area_size,
            )?;
        }
        // Capture optimized LLVM IR for output if requested and not writing to files
        let captured_llvm_ir = if self.capture_llvm_ir && self.output_config.is_none() {
            Some(self.module.print_to_string().to_string())
        } else {
            None
        };
        self.verify().map_err(|error| {
            anyhow::anyhow!(
                "{} code optimized LLVM IR verification: {error}",
                self.code_segment,
            )
        })?;
        run_optimize_verify.borrow_mut().finish();

        let assembly_buffer = if output_assembly
            || self
                .output_config
                .as_ref()
                .is_some_and(|output_config| output_config.output_assembly)
        {
            let run_emit_llvm_assembly = profiler.start_evm_translation_unit(
                contract_path,
                Some(self.code_segment),
                "EmitLLVMAssembly",
                optimizer_mode.as_str(),
                spill_area_size,
            );
            let module_assembly_emitter = self.module.clone();
            let assembly_buffer = target_machine
                .write_to_memory_buffer(
                    &module_assembly_emitter,
                    inkwell::targets::FileType::Assembly,
                )
                .map_err(|error| anyhow::anyhow!("assembly emitting: {error}"))?;
            if let Some(output_config) = self.output_config.as_ref() {
                let assembly_text = String::from_utf8_lossy(assembly_buffer.as_slice());
                output_config.dump_assembly(
                    contract_path,
                    assembly_text.as_ref(),
                    is_size_fallback,
                    spill_area_size,
                )?;
            }
            run_emit_llvm_assembly.borrow_mut().finish();
            Some(assembly_buffer)
        } else {
            None
        };
        let assembly = assembly_buffer
            .map(|assembly_buffer| String::from_utf8_lossy(assembly_buffer.as_slice()).to_string());

        if !output_bytecode {
            return Ok(EVMBuild::new(
                None,
                assembly,
                captured_llvm_ir_unoptimized,
                captured_llvm_ir,
                None,
                is_size_fallback,
                vec![],
            ));
        }

        let run_emit_bytecode = profiler.start_evm_translation_unit(
            contract_path,
            Some(self.code_segment),
            "EmitBytecode",
            optimizer_mode.as_str(),
            spill_area_size,
        );
        let bytecode_buffer = target_machine
            .write_to_memory_buffer(&self.module, inkwell::targets::FileType::Object)
            .map_err(|error| anyhow::anyhow!("{} bytecode emitting: {error}", self.code_segment))?;
        run_emit_bytecode.borrow_mut().finish();

        let immutables = match self.code_segment {
            solx_utils::CodeSegment::Deploy => None,
            solx_utils::CodeSegment::Runtime => Some(bytecode_buffer.get_immutables_evm()),
        };

        let mut warnings = Vec::with_capacity(1);
        let bytecode_size = bytecode_buffer.as_slice().len();
        if bytecode_size > self.code_segment.size_limit() {
            if needs_size_fallback {
                crate::codegen::IS_SIZE_FALLBACK.store(true, std::sync::atomic::Ordering::Relaxed);
                let mut size_fallback_settings = OptimizerSettings::size();
                size_fallback_settings.metadata_size = self.optimizer.settings().metadata_size;
                self.optimizer = Optimizer::new(size_fallback_settings);
                self.module = module_size_fallback
                    .expect("cloned when the settings enable the size fallback");
                for function in self.module.get_functions() {
                    self.set_size_attributes(function);
                }
                return self.build(output_assembly, output_bytecode, true, profiler);
            } else {
                warnings.push(solx_utils::Warning::code_size(
                    self.code_segment,
                    bytecode_size,
                ))
            };
        }

        Ok(EVMBuild::new(
            Some(bytecode_buffer.as_slice().to_vec()),
            assembly,
            captured_llvm_ir_unoptimized,
            captured_llvm_ir,
            immutables,
            is_size_fallback,
            warnings,
        ))
    }

    ///
    /// Verifies the current LLVM IR module.
    ///
    pub fn verify(&self) -> anyhow::Result<()> {
        self.module
            .verify()
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    ///
    /// Enables LLVM IR capture for output.
    ///
    pub fn set_capture_llvm_ir(&mut self, capture: bool) {
        self.capture_llvm_ir = capture;
    }

    ///
    /// Sets the size optimization attributes.
    ///
    fn set_size_attributes(&self, function: inkwell::values::FunctionValue<'ctx>) {
        for attribute in [Attribute::OptimizeForSize, Attribute::MinSize] {
            function.add_attribute(
                inkwell::attributes::AttributeLoc::Function,
                self.llvm.create_enum_attribute(attribute as u32, 0),
            );
        }
    }
}
