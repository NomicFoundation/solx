//!
//! Solidity compiler arguments.
//!

use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use clap::Parser;
use path_slash::PathExt;

///
/// Solidity compiler arguments.
///
#[derive(Debug, Parser)]
#[command(about, long_about = "LLVM-based Solidity compiler for the EVM")]
pub struct Arguments {
    /// Print the version and exit.
    #[arg(long)]
    pub version: bool,

    /// Specify the input paths and remappings.
    /// If an argument contains a '=', it is considered a remapping.
    pub inputs: Vec<String>,

    //
    // Input Options
    //
    /// Set the given path as the root of the source tree instead of the root of the filesystem.
    /// Passed to `solc` without changes.
    #[arg(long, help_heading = "Input Options")]
    pub base_path: Option<String>,

    /// Make an additional source directory available to the default import callback.
    /// Can be used multiple times. Can only be used if the base path has a non-empty value.
    /// Passed to `solc` without changes.
    #[arg(long, num_args = 1.., help_heading = "Input Options")]
    pub include_path: Vec<String>,

    /// Allow a given path for imports. A list of paths can be supplied by separating them with a comma.
    /// Passed to `solc` without changes.
    #[arg(long, help_heading = "Input Options")]
    pub allow_paths: Option<String>,

    /// Switch to Yul mode.
    /// Only one input Yul file is allowed.
    /// Cannot be used with standard JSON mode.
    #[arg(long, alias = "strict-assembly", help_heading = "Input Options")]
    pub yul: bool,

    /// Switch to LLVM IR mode.
    /// Only one input LLVM IR file is allowed.
    /// Cannot be used with standard JSON mode.
    /// Use this mode at your own risk, as LLVM IR input validation is not implemented.
    #[arg(long, help_heading = "Input Options")]
    pub llvm_ir: bool,

    /// Switch to standard JSON input/output mode. Read from stdin or specified file, write the result to stdout.
    /// This is the default used by the Hardhat plugin.
    #[arg(long, help_heading = "Input Options")]
    pub standard_json: Option<Option<String>>,

    /// Specify addresses of deployable libraries. Syntax: `<libraryFullPath1>=<address1> ... <libraryFullPathN>=<addressN>`.
    /// Addresses are interpreted as hexadecimal strings prefixed with `0x`.
    #[arg(short, long, num_args = 1.., help_heading = "Input Options")]
    pub libraries: Vec<String>,

    //
    // Output Options
    //
    /// Create one file per component and contract/file at the specified directory, if given.
    #[arg(short, long, help_heading = "Output Options")]
    pub output_dir: Option<PathBuf>,

    /// Overwrite existing files (used together with -o).
    #[arg(long, help_heading = "Output Options")]
    pub overwrite: bool,

    //
    // Output Selection
    //
    /// Emit bytecode of the compiled contracts.
    #[arg(long = "bin", help_heading = "Output Selection")]
    pub output_bytecode: bool,

    /// Emit deployed bytecode of the compiled contracts.
    #[arg(long = "bin-runtime", help_heading = "Output Selection")]
    pub output_bytecode_runtime: bool,

    /// Emit assembly of the compiled contracts.
    #[arg(long = "asm", help_heading = "Output Selection")]
    pub output_assembly: bool,

    /// Emit debug info of the compiled contracts.
    #[arg(long = "debug-info", help_heading = "Output Selection")]
    pub output_debug_info: bool,

    /// Emit runtime bytecode debug info of the compiled contracts.
    #[arg(long = "debug-info-runtime", help_heading = "Output Selection")]
    pub output_debug_info_runtime: bool,

    /// Emit metadata of the compiled project.
    #[arg(long = "metadata", help_heading = "Output Selection")]
    pub output_metadata: bool,

    /// Emit ABI specification of the compiled project.
    #[arg(long = "abi", help_heading = "Output Selection")]
    pub output_abi: bool,

    /// Emit function signature hashes of the compiled project.
    #[arg(long = "hashes", help_heading = "Output Selection")]
    pub output_hashes: bool,

    /// Emit user documentation of the compiled project.
    #[arg(long = "userdoc", help_heading = "Output Selection")]
    pub output_userdoc: bool,

    /// Emit developer documentation of the compiled project.
    #[arg(long = "devdoc", help_heading = "Output Selection")]
    pub output_devdoc: bool,

    /// Emit storage layout of the compiled project.
    #[arg(long = "storage-layout", help_heading = "Output Selection")]
    pub output_storage_layout: bool,

    /// Emit storage layout of the compiled project.
    #[arg(long = "transient-storage-layout", help_heading = "Output Selection")]
    pub output_transient_storage_layout: bool,

    /// Emit AST of the compiled project.
    #[arg(long = "ast-json", help_heading = "Output Selection")]
    pub output_ast_json: bool,

    /// Emit solc's EVM assembly of the compiled project.
    #[arg(long = "asm-solc-json", help_heading = "Output Selection")]
    pub output_asm_solc_json: bool,

    /// Emit solc's Yul IR of the compiled project.
    #[arg(long = "ir", alias = "ir-optimized", help_heading = "Output Selection")]
    pub output_ir: bool,

    /// Emit solx's compilation pipeline benchmarks.
    #[arg(long = "benchmarks", help_heading = "Output Selection")]
    pub output_benchmarks: bool,

    /// Emit EVM legacy assembly (intermediate representation from solc).
    /// Can be used with --output-dir to write .evmla files.
    #[arg(long = "evmla", help_heading = "Output Selection")]
    pub output_evmla: bool,

    /// Emit Ethereal IR (intermediate representation between EVM assembly and LLVM IR).
    /// Can be used with --output-dir to write .ethir files.
    #[arg(long = "ethir", help_heading = "Output Selection")]
    pub output_ethir: bool,

    /// Emit LLVM IR (both unoptimized and optimized).
    /// Can be used with --output-dir to write .ll files.
    #[arg(long = "emit-llvm", help_heading = "Output Selection")]
    pub output_llvm_ir: bool,

    //
    // Compilation Settings
    //
    /// EVM version `solc` will produce Yul or EVM assembly for.
    /// The default is chosen by `solc`.
    #[arg(long, help_heading = "Compilation Settings")]
    pub evm_version: Option<solx_utils::EVMVersion>,

    /// Enable the `solc` IR codegen.
    #[arg(long, help_heading = "Compilation Settings")]
    pub via_ir: bool,

    /// Sets the number of threads, where each thread compiles its own translation unit in a child process.
    #[arg(short, long, help_heading = "Compilation Settings")]
    pub threads: Option<usize>,

    //
    // Optimization
    //
    /// Set the optimization parameter -O[0 | 1 | 2 | 3 | s | z].
    /// Use `3` for best performance and `z` for minimal size.
    #[arg(short = 'O', long, help_heading = "Optimization")]
    pub optimization: Option<char>,

    /// Try to recompile with -Oz if the bytecode is too large.
    #[arg(long = "optimization-size-fallback", help_heading = "Optimization")]
    pub size_fallback: bool,

    /// Pass arbitrary space-separated options to LLVM.
    /// The argument must be a single-quoted string following a `=` separator.
    /// Example: `--llvm-options='arg1 arg2 arg3 ... argN'`.
    #[arg(long, help_heading = "Optimization")]
    pub llvm_options: Option<String>,

    //
    // Metadata
    //
    /// Set the metadata hash type.
    /// Available types: `none`, `ipfs`.
    /// The default is `ipfs`.
    #[arg(long, help_heading = "Metadata")]
    pub metadata_hash: Option<solx_utils::MetadataHashType>,

    /// Sets the literal content flag for contract metadata.
    /// If enabled, the metadata will contain the literal content of the source files.
    #[arg(long, help_heading = "Metadata")]
    pub metadata_literal: bool,

    /// Turn off CBOR metadata at the end of bytecode.
    #[arg(long, help_heading = "Metadata")]
    pub no_cbor_metadata: bool,

    /// Turn off the default `solc` import resolution callback.
    #[arg(long, help_heading = "Metadata")]
    pub no_import_callback: bool,

    //
    // Debug Options
    //
    /// Set the verify-each option in LLVM.
    /// Only for testing and debugging.
    #[arg(long, help_heading = "Debug Options")]
    pub llvm_verify_each: bool,

    /// Set the debug-logging option in LLVM.
    /// Only for testing and debugging.
    #[arg(long, help_heading = "Debug Options")]
    pub llvm_debug_logging: bool,

    /// Run this process recursively and provide JSON input to compile a single contract.
    /// Only for usage from within the compiler.
    #[arg(long, hide = true)]
    pub recursive_process: bool,
}

impl Arguments {
    ///
    /// Validates the arguments.
    ///
    pub fn validate(&self) -> Arc<Mutex<Vec<solx_standard_json::OutputError>>> {
        let mut messages = vec![];

        if self.recursive_process {
            if std::env::args().count() > 3 {
                messages.push(solx_standard_json::OutputError::new_error(
                    "No other options are allowed while running in the recursive process mode.",
                ));
            }
            return Arc::new(Mutex::new(messages));
        }

        if self.version {
            if std::env::args().count() > 2 {
                messages.push(solx_standard_json::OutputError::new_error(
                    "No other options are allowed while getting the compiler version.",
                ));
            }
            return Arc::new(Mutex::new(messages));
        }

        if self.standard_json.is_none() && self.inputs.is_empty() {
            messages.push(solx_standard_json::OutputError::new_error(
                format!("No input files given. For standard input, specify `{}` explicitly, or visit `--help` to see all options.", solx_standard_json::InputSource::STDIN_INPUT_IDENTIFIER).as_str(),
            ));
        }

        let modes_count = [self.yul, self.llvm_ir, self.standard_json.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        if modes_count > 1 {
            messages.push(solx_standard_json::OutputError::new_error(
                "Only one mode is allowed at the same time: Yul, LLVM IR, standard JSON.",
            ));
        }

        if self.yul || self.llvm_ir {
            if self.base_path.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "`base-path` is only allowed in Solidity mode.",
                ));
            }
            if !self.include_path.is_empty() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "`include-path` is only allowed in Solidity mode.",
                ));
            }
            if self.allow_paths.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "`allow-paths` is only allowed in Solidity mode.",
                ));
            }

            if self.output_abi
                || self.output_hashes
                || self.output_userdoc
                || self.output_devdoc
                || self.output_storage_layout
                || self.output_transient_storage_layout
                || self.output_ast_json
                || self.output_asm_solc_json
                || self.output_ir
                || self.output_debug_info
                || self.output_debug_info_runtime
                || self.output_benchmarks
            {
                messages.push(solx_standard_json::OutputError::new_error(
                    "ABI, hashes, userdoc, devdoc, storage layout, transient storage layout, AST, EVM assembly, Yul, debug info, benchmarks can be only emitted for Solidity contracts.",
                ));
            }

            if self.evm_version.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "EVM version is only allowed in Solidity mode.",
                ));
            }

            if self.via_ir {
                messages.push(solx_standard_json::OutputError::new_error(
                    "IR codegen settings are only available in Solidity mode.",
                ));
            }
        }

        if self.standard_json.is_some() {
            if self.output_bytecode
                || self.output_bytecode_runtime
                || self.output_assembly
                || self.output_debug_info
                || self.output_debug_info_runtime
                || self.output_metadata
                || self.output_abi
                || self.output_hashes
                || self.output_userdoc
                || self.output_devdoc
                || self.output_storage_layout
                || self.output_transient_storage_layout
                || self.output_ast_json
                || self.output_asm_solc_json
                || self.output_ir
                || self.output_benchmarks
                || self.output_evmla
                || self.output_ethir
                || self.output_llvm_ir
            {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Cannot output data outside of JSON in standard JSON mode.",
                ));
            }

            if !self.inputs.is_empty() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Input files must be passed via standard JSON input.",
                ));
            }
            if !self.libraries.is_empty() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Libraries must be passed via standard JSON input.",
                ));
            }

            if self.via_ir {
                messages.push(solx_standard_json::OutputError::new_error(
                    "IR codegen must be passed via standard JSON input.",
                ));
            }
            if self.evm_version.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "EVM version must be passed via standard JSON input.",
                ));
            }

            if self.output_dir.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Output directory cannot be used in standard JSON mode.",
                ));
            }
            if self.overwrite {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Overwriting flag cannot be used in standard JSON mode.",
                ));
            }
            if self.optimization.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "LLVM optimizations must be specified in standard JSON input settings.",
                ));
            }
            if self.size_fallback {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Size optimization fallback must be specified in standard JSON input settings.",
                ));
            }
            if self.llvm_options.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "LLVM options must be specified in standard JSON input settings.",
                ));
            }
            if self.metadata_hash.is_some() {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Metadata hash mode must be specified in standard JSON input settings.",
                ));
            }
            if self.metadata_literal {
                messages.push(solx_standard_json::OutputError::new_error(
                    "Metadata literal content flag must be specified in standard JSON input settings.",
                ));
            }
        }

        Arc::new(Mutex::new(messages))
    }

    ///
    /// Returns remappings from input paths.
    ///
    pub fn split_input_files_and_remappings(
        &self,
    ) -> anyhow::Result<(Vec<PathBuf>, BTreeSet<String>)> {
        let mut input_files = Vec::with_capacity(self.inputs.len());
        let mut remappings = BTreeSet::new();

        for input in self.inputs.iter() {
            if input.contains('=') {
                let mut parts = Vec::with_capacity(2);
                for path in input.trim().split('=') {
                    let path = PathBuf::from(path);
                    parts.push(
                        Self::path_to_posix(path.as_path())?
                            .to_string_lossy()
                            .to_string(),
                    );
                }
                if parts.len() != 2 {
                    anyhow::bail!(
                        "Invalid remapping `{input}`: expected two parts separated by '='."
                    );
                }
                remappings.insert(parts.join("="));
            } else {
                let path = PathBuf::from(input.trim());
                let path = Self::path_to_posix(path.as_path())?;
                input_files.push(path);
            }
        }

        Ok((input_files, remappings))
    }

    ///
    /// Normalizes an input path by converting it to POSIX format.
    ///
    fn path_to_posix(path: &Path) -> anyhow::Result<PathBuf> {
        let path = path
            .to_slash()
            .ok_or_else(|| anyhow::anyhow!("Input path {path:?} POSIX conversion error."))?
            .to_string();
        let path = PathBuf::from(path.as_str());
        Ok(path)
    }
}
