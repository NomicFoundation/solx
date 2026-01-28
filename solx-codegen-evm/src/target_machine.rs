//!
//! EVM target machine.
//!

use crate::optimizer::settings::Settings as OptimizerSettings;

///
/// EVM target machine.
///
#[derive(Debug)]
pub struct TargetMachine {
    /// The LLVM target machine reference.
    target_machine: inkwell::targets::TargetMachine,
    /// The optimizer settings.
    optimizer_settings: OptimizerSettings,
}

impl TargetMachine {
    /// The EVM target identifier.
    const TARGET: solx_utils::Target = solx_utils::Target::EVM;

    ///
    /// A shortcut constructor.
    ///
    /// Supported LLVM options:
    /// `-evm-stack-region-size <value>`
    /// `-evm-stack-region-offset <value>`
    /// `-evm-metadata-size <value>`
    ///
    pub fn new(
        optimizer_settings: &OptimizerSettings,
        llvm_options: &[String],
    ) -> anyhow::Result<Self> {
        let mut arguments = Vec::with_capacity(1 + llvm_options.len());
        arguments.push(Self::TARGET.to_string());
        arguments.extend_from_slice(llvm_options);
        if let Some(size) = optimizer_settings.spill_area_size {
            arguments.push(format!(
                "-evm-stack-region-offset={}",
                crate::r#const::SOLC_USER_MEMORY_OFFSET
            ));
            arguments.push(format!("-evm-stack-region-size={size}"));
        }
        if let Some(size) = optimizer_settings.metadata_size {
            arguments.push(format!("-evm-metadata-size={size}"));
        }
        if arguments.len() > 1 {
            let arguments: Vec<&str> = arguments.iter().map(|argument| argument.as_str()).collect();
            inkwell::support::parse_command_line_options(arguments.as_slice(), "LLVM options");
        }

        let target_machine = inkwell::targets::Target::from_name(Self::TARGET.to_string().as_str())
            .ok_or_else(|| anyhow::anyhow!("LLVM target machine `{}` not found", Self::TARGET))?
            .create_target_machine(
                &inkwell::targets::TargetTriple::create(Self::TARGET.triple()),
                "",
                "",
                optimizer_settings.level_back_end,
                inkwell::targets::RelocMode::Default,
                inkwell::targets::CodeModel::Default,
            )
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "LLVM target machine `{}` initialization error",
                    Self::TARGET
                )
            })?;

        Ok(Self {
            target_machine,
            optimizer_settings: optimizer_settings.to_owned(),
        })
    }

    ///
    /// Sets the target-specific data in the module.
    ///
    pub fn set_target_data(&self, module: &inkwell::module::Module) {
        module.set_triple(&self.target_machine.get_triple());
        module.set_data_layout(&self.target_machine.get_target_data().get_data_layout());
    }

    ///
    /// Sets the assembly printer verbosity.
    ///
    pub fn set_asm_verbosity(&self, verbosity: bool) {
        self.target_machine.set_asm_verbosity(verbosity);
    }

    ///
    /// Writes the LLVM module to a memory buffer.
    ///
    pub fn write_to_memory_buffer(
        &self,
        module: &inkwell::module::Module,
        file_type: inkwell::targets::FileType,
    ) -> Result<inkwell::memory_buffer::MemoryBuffer, inkwell::support::LLVMString> {
        self.target_machine
            .write_to_memory_buffer(module, file_type)
    }

    ///
    /// Writes the LLVM module and debug info to memory buffers.
    ///
    pub fn write_to_memory_buffer_with_debug_info(
        &self,
        module: &inkwell::module::Module,
        file_type: inkwell::targets::FileType,
    ) -> Result<
        (
            inkwell::memory_buffer::MemoryBuffer,
            inkwell::memory_buffer::MemoryBuffer,
        ),
        inkwell::support::LLVMString,
    > {
        self.target_machine
            .write_to_memory_buffer_with_dbg(module, file_type)
    }

    ///
    /// Runs the optimization passes on `module`.
    ///
    pub fn run_optimization_passes(
        &self,
        module: &inkwell::module::Module,
        passes: &str,
    ) -> Result<(), inkwell::support::LLVMString> {
        let pass_builder_options = inkwell::passes::PassBuilderOptions::create();
        pass_builder_options.set_verify_each(self.optimizer_settings.is_verify_each_enabled);
        pass_builder_options.set_debug_logging(self.optimizer_settings.is_debug_logging_enabled);

        module.run_passes(passes, &self.target_machine, pass_builder_options)
    }

    ///
    /// Returns the target triple.
    ///
    pub fn get_triple(&self) -> inkwell::targets::TargetTriple {
        self.target_machine.get_triple()
    }

    ///
    /// Returns the target data.
    ///
    pub fn get_target_data(&self) -> inkwell::targets::TargetData {
        self.target_machine.get_target_data()
    }
}
