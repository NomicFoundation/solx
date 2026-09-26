//!
//! The output configuration for IR artifacts.
//!

pub mod ir_type;

use std::path::Path;
use std::path::PathBuf;

use self::ir_type::IRType;

///
/// The output configuration for IR artifacts.
///
/// Controls which intermediate representations are written to disk during compilation.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputConfig {
    /// The directory to write the IRs to.
    pub output_directory: PathBuf,
    /// Whether to overwrite existing files.
    #[serde(default)]
    pub overwrite: bool,
    /// Whether to output LLVM IR.
    #[serde(default)]
    pub output_llvm_ir: bool,
    /// Whether to output LLVM assembly.
    #[serde(default)]
    pub output_assembly: bool,
}

impl OutputConfig {
    ///
    /// A shortcut constructor for debug mode (all outputs enabled, overwrite on).
    ///
    pub fn new_debug(output_directory: PathBuf) -> Self {
        Self {
            output_directory,
            overwrite: true,
            output_llvm_ir: true,
            output_assembly: true,
        }
    }

    ///
    /// A shortcut constructor with selective outputs.
    ///
    pub fn new(
        output_directory: PathBuf,
        overwrite: bool,
        output_llvm_ir: bool,
        output_assembly: bool,
    ) -> Self {
        Self {
            output_directory,
            overwrite,
            output_llvm_ir,
            output_assembly,
        }
    }

    ///
    /// Create a subdirectory and return a copy of `OutputConfig` pointing there.
    ///
    pub fn create_subdirectory(&self, directory_name: &str) -> anyhow::Result<Self> {
        let sanitized_name = solx_utils::ContractName::sanitize_path(directory_name);
        let subdirectory_path = self.output_directory.join(sanitized_name.as_str());
        std::fs::create_dir_all(subdirectory_path.as_path())?;
        Ok(Self {
            output_directory: subdirectory_path,
            overwrite: self.overwrite,
            output_llvm_ir: self.output_llvm_ir,
            output_assembly: self.output_assembly,
        })
    }

    ///
    /// Builds a suffix string from size fallback and spill area parameters.
    ///
    fn build_suffix(is_size_fallback: bool, spill_area_size: Option<u64>) -> Option<String> {
        let mut suffix = String::new();
        if is_size_fallback {
            suffix.push_str("size_fallback");
        }
        if let Some(size) = spill_area_size {
            if !suffix.is_empty() {
                suffix.push('.');
            }
            suffix.push_str(format!("s{size}").as_str());
        }
        if suffix.is_empty() {
            None
        } else {
            Some(suffix)
        }
    }

    ///
    /// Dumps the unoptimized LLVM IR.
    ///
    pub fn dump_llvm_ir_unoptimized(
        &self,
        contract_path: &str,
        module: &inkwell::module::Module,
        is_size_fallback: bool,
        spill_area_size: Option<u64>,
    ) -> anyhow::Result<()> {
        if !self.output_llvm_ir {
            return Ok(());
        }
        let llvm_code = module.print_to_string().to_string();

        let mut suffix = "unoptimized".to_owned();
        if let Some(extra) = Self::build_suffix(is_size_fallback, spill_area_size) {
            suffix.push('.');
            suffix.push_str(extra.as_str());
        }

        let mut file_path = self.output_directory.to_owned();
        let full_file_name =
            Self::full_file_name(contract_path, Some(suffix.as_str()), IRType::LLVM);
        file_path.push(full_file_name);
        self.write_file(file_path.as_path(), llvm_code)?;

        Ok(())
    }

    ///
    /// Dumps the optimized LLVM IR.
    ///
    pub fn dump_llvm_ir_optimized(
        &self,
        contract_path: &str,
        module: &inkwell::module::Module,
        is_size_fallback: bool,
        spill_area_size: Option<u64>,
    ) -> anyhow::Result<()> {
        if !self.output_llvm_ir {
            return Ok(());
        }
        let llvm_code = module.print_to_string().to_string();

        let mut suffix = "optimized".to_owned();
        if let Some(extra) = Self::build_suffix(is_size_fallback, spill_area_size) {
            suffix.push('.');
            suffix.push_str(extra.as_str());
        }

        let mut file_path = self.output_directory.to_owned();
        let full_file_name =
            Self::full_file_name(contract_path, Some(suffix.as_str()), IRType::LLVM);
        file_path.push(full_file_name);
        self.write_file(file_path.as_path(), llvm_code)?;

        Ok(())
    }

    ///
    /// Dumps the assembly.
    ///
    pub fn dump_assembly(
        &self,
        contract_path: &str,
        code: &str,
        is_size_fallback: bool,
        spill_area_size: Option<u64>,
    ) -> anyhow::Result<()> {
        if !self.output_assembly {
            return Ok(());
        }
        let suffix = Self::build_suffix(is_size_fallback, spill_area_size);

        let mut file_path = self.output_directory.to_owned();
        let full_file_name =
            Self::full_file_name(contract_path, suffix.as_deref(), IRType::EVMAssembly);
        file_path.push(full_file_name);
        self.write_file(file_path.as_path(), code)?;

        Ok(())
    }

    ///
    /// Writes data to the file, respecting the `overwrite` flag.
    ///
    fn write_file<C: AsRef<[u8]>>(&self, output_path: &Path, data: C) -> anyhow::Result<()> {
        if output_path.exists() && !self.overwrite {
            anyhow::bail!(
                "Refusing to overwrite an existing file {output_path:?} (use --overwrite to force)."
            );
        }
        std::fs::write(output_path, data)
            .map_err(|error| anyhow::anyhow!("File {output_path:?} writing: {error}"))?;
        Ok(())
    }

    ///
    /// Creates a full file name, given the contract full path, suffix, and extension.
    ///
    fn full_file_name(contract_path: &str, suffix: Option<&str>, ir_type: IRType) -> String {
        let mut full_file_name = solx_utils::ContractName::sanitize_path(contract_path);

        if let Some(suffix) = suffix {
            full_file_name.push('.');
            full_file_name.push_str(suffix);
        }
        full_file_name.push('.');
        full_file_name.push_str(ir_type.file_extension());
        full_file_name
    }
}
