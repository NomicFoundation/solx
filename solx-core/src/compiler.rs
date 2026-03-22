//! Compilation pipeline entry points.

use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use solx_standard_json::CollectableError;

use crate::Arguments;
use crate::DEFAULT_EXECUTABLE_NAME;
use crate::DEFAULT_PACKAGE_DESCRIPTION;
use crate::EVMBuild;
use crate::Frontend;
use crate::Project;
use crate::WORKER_THREAD_STACK_SIZE;

/// Orchestrates compilation from Solidity sources to EVM bytecode.
pub struct Compiler<'arguments> {
    /// The compilation arguments.
    arguments: &'arguments Arguments,
}

impl<'arguments> Compiler<'arguments> {
    /// Creates a new compiler instance from the given arguments.
    pub fn new(arguments: &'arguments Arguments) -> Self {
        Self { arguments }
    }

    ///
    /// Initialize the compiler runtime: rayon thread pool, LLVM stack trace, and
    /// EVM target.
    ///
    /// If `arguments.recursive_process` is set, runs the subprocess handler and
    /// returns `Ok(true)` -- the caller should return immediately.
    /// Otherwise returns `Ok(false)`.
    ///
    pub fn initialize(&self) -> anyhow::Result<bool> {
        let mut thread_pool_builder = rayon::ThreadPoolBuilder::new();
        if let Some(threads) = self.arguments.threads {
            thread_pool_builder = thread_pool_builder.num_threads(threads);
        }
        thread_pool_builder
            .stack_size(WORKER_THREAD_STACK_SIZE)
            .build_global()
            .expect("rayon thread pool parameters are valid");

        inkwell::support::enable_llvm_pretty_stack_trace();
        solx_codegen_evm::initialize_target();

        if self.arguments.recursive_process {
            crate::run_subprocess()?;
            return Ok(true);
        }

        Ok(false)
    }

    ///
    /// The `main` function that implements the core CLI application logic.
    ///
    pub fn run<F>(
        &self,
        frontend: F,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
    ) -> anyhow::Result<()>
    where
        F: Frontend,
    {
        if self.initialize()? {
            return Ok(());
        }

        let (input_files, remappings) = self.arguments.split_input_files_and_remappings()?;

        let optimizer_settings = self.arguments.optimizer_settings()?;
        let output_selection = self.arguments.output_selection();
        let llvm_options = self.arguments.llvm_options();

        let output_config = self.arguments.output_config()?;

        let metadata_hash_type = self
            .arguments
            .metadata_hash
            .unwrap_or(solx_utils::MetadataHashType::IPFS);
        let append_cbor = !self.arguments.no_cbor_metadata;
        let use_import_callback = !self.arguments.no_import_callback;

        let build = if self.arguments.yul {
            self.yul_to_evm(
                &frontend,
                input_files.as_slice(),
                self.arguments.libraries.as_slice(),
                &output_selection,
                messages,
                self.arguments.evm_version,
                metadata_hash_type,
                append_cbor,
                optimizer_settings,
                llvm_options,
                output_config,
            )
        } else if self.arguments.llvm_ir {
            self.llvm_ir_to_evm(
                input_files.as_slice(),
                self.arguments.libraries.as_slice(),
                &output_selection,
                messages,
                self.arguments.evm_version,
                metadata_hash_type,
                append_cbor,
                optimizer_settings,
                llvm_options,
                output_config,
            )
        } else if let Some(ref standard_json) = self.arguments.standard_json {
            return self.standard_json_evm(
                frontend,
                standard_json.as_ref().map(PathBuf::from),
                messages,
                self.arguments.base_path.clone(),
                self.arguments.include_path.clone(),
                self.arguments.allow_paths.clone(),
                use_import_callback,
                output_config,
            );
        } else {
            self.standard_output_evm(
                frontend,
                input_files.as_slice(),
                self.arguments.libraries.as_slice(),
                &output_selection,
                messages,
                self.arguments.evm_version,
                self.arguments.via_ir,
                metadata_hash_type,
                self.arguments.metadata_literal,
                append_cbor,
                self.arguments.base_path.clone(),
                self.arguments.include_path.clone(),
                self.arguments.allow_paths.clone(),
                use_import_callback,
                remappings,
                optimizer_settings,
                llvm_options,
                output_config,
            )
        }?;

        if output_selection.is_empty() {
            writeln!(
                std::io::stdout(),
                "Compiler run successful. No output generated."
            )?;
            return Ok(());
        }

        if let Some(ref output_directory) = self.arguments.output_dir {
            build.write_to_directory(
                output_directory,
                &output_selection,
                self.arguments.overwrite,
            )?;
        } else {
            build.write_to_terminal(&output_selection)?;
        }

        Ok(())
    }

    ///
    /// Runs the Yul mode for the EVM target.
    ///
    pub fn yul_to_evm<F>(
        &self,
        frontend: &F,
        paths: &[PathBuf],
        libraries: &[String],
        output_selection: &solx_standard_json::InputSelection,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<EVMBuild>
    where
        F: Frontend,
    {
        if output_selection.is_debug_info_set_for_any() {
            anyhow::bail!(solx_standard_json::OutputError::new_error(
                "Debug info is only supported for Solidity source code input."
            ));
        }

        let libraries = solx_utils::Libraries::try_from(libraries)?;
        let linker_symbols = libraries.as_linker_symbols()?;

        frontend.validate_yul_paths(paths, libraries.clone())?;

        let project = Project::try_from_yul_paths(
            frontend.version(),
            paths,
            libraries,
            output_selection,
            None,
            output_config.as_ref(),
        )?;

        let mut build = project.compile_to_evm(
            messages,
            output_selection,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;
        build.take_and_write_warnings();
        build.check_errors()?;

        Ok(if output_selection.is_bytecode_set_for_any() {
            let mut build = build.link(linker_symbols);
            build.take_and_write_warnings();
            build.check_errors()?;
            build
        } else {
            build
        })
    }

    ///
    /// Runs the LLVM IR mode for the EVM target.
    ///
    pub fn llvm_ir_to_evm(
        &self,
        paths: &[PathBuf],
        libraries: &[String],
        output_selection: &solx_standard_json::InputSelection,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        metadata_hash_type: solx_utils::MetadataHashType,
        append_cbor: bool,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<EVMBuild> {
        if output_selection.is_debug_info_set_for_any() {
            anyhow::bail!(solx_standard_json::OutputError::new_error(
                "Debug info is only supported for Solidity source code input."
            ));
        }

        let libraries = solx_utils::Libraries::try_from(libraries)?;
        let linker_symbols = libraries.as_linker_symbols()?;

        let project = Project::try_from_llvm_ir_paths(paths, libraries, output_selection, None)?;

        let mut build = project.compile_to_evm(
            messages,
            output_selection,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings,
            llvm_options,
            output_config,
        )?;
        build.take_and_write_warnings();
        build.check_errors()?;

        Ok(if output_selection.is_bytecode_set_for_any() {
            let mut build = build.link(linker_symbols);
            build.take_and_write_warnings();
            build.check_errors()?;
            build
        } else {
            build
        })
    }

    ///
    /// Runs the standard output mode for the EVM target.
    ///
    pub fn standard_output_evm<F>(
        &self,
        frontend: F,
        paths: &[PathBuf],
        libraries: &[String],
        output_selection: &solx_standard_json::InputSelection,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        evm_version: Option<solx_utils::EVMVersion>,
        via_ir: bool,
        metadata_hash_type: solx_utils::MetadataHashType,
        metadata_literal: bool,
        append_cbor: bool,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
        use_import_callback: bool,
        remappings: BTreeSet<String>,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<EVMBuild>
    where
        F: Frontend,
    {
        let mut profiler = solx_codegen_evm::Profiler::default();

        let mut solc_input = solx_standard_json::Input::try_from_solidity_paths(
            paths,
            libraries,
            remappings,
            solx_standard_json::InputOptimizer::default(),
            evm_version,
            via_ir,
            output_selection,
            solx_standard_json::InputMetadata::new(
                metadata_literal,
                append_cbor,
                metadata_hash_type,
            ),
            llvm_options.clone(),
        )?;

        let run_solc_standard_json = profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
        let mut solc_output = frontend.standard_json(
            &mut solc_input,
            use_import_callback,
            base_path.as_deref(),
            include_paths.as_slice(),
            allow_paths,
        )?;
        run_solc_standard_json.borrow_mut().finish();
        solc_output.take_and_write_warnings();
        solc_output.check_errors()?;

        let linker_symbols = solc_input.settings.libraries.as_linker_symbols()?;
        solc_input.resolve_sources()?;
        let debug_info = solc_output.get_debug_info(&solc_input.sources);

        let run_solx_project = profiler.start_pipeline_element("solx_Solidity_IR_Analysis");
        let project = Project::try_from_solidity_output(
            frontend.version(),
            solc_input.settings.libraries.clone(),
            via_ir,
            &mut solc_output,
            Some(debug_info),
            output_config.as_ref(),
        )?;
        run_solx_project.borrow_mut().finish();
        solc_output.take_and_write_warnings();
        solc_output.check_errors()?;

        let run_solx_compile = profiler.start_pipeline_element("solx_Compilation");
        let mut build = project.compile_to_evm(
            messages,
            &solc_input.settings.output_selection,
            evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings.clone(),
            llvm_options,
            output_config.clone(),
        )?;
        run_solx_compile.borrow_mut().finish();
        build.take_and_write_warnings();
        build.check_errors()?;

        let mut build = if solc_input
            .settings
            .output_selection
            .is_bytecode_set_for_any()
        {
            let run_solx_link = profiler.start_pipeline_element("solx_Linking");
            let mut build = build.link(linker_symbols);
            run_solx_link.borrow_mut().finish();
            build.take_and_write_warnings();
            build.check_errors()?;
            build
        } else {
            build
        };
        build.benchmarks = profiler.to_vec();
        Ok(build)
    }

    ///
    /// Runs the standard JSON mode for the EVM target.
    ///
    pub fn standard_json_evm<F>(
        &self,
        frontend: F,
        json_path: Option<PathBuf>,
        messages: Arc<Mutex<Vec<solx_standard_json::OutputError>>>,
        base_path: Option<String>,
        include_paths: Vec<String>,
        allow_paths: Option<String>,
        use_import_callback: bool,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<()>
    where
        F: Frontend,
    {
        let mut solc_input = solx_standard_json::Input::try_from(json_path.as_deref())?;
        let language = solc_input.language;
        let via_ir = solc_input.settings.via_ir;
        let linker_symbols = solc_input.settings.libraries.as_linker_symbols()?;

        let optimizer_settings = solx_codegen_evm::OptimizerSettings::try_from_standard_json(
            solc_input.settings.optimizer.mode,
            solc_input.settings.optimizer.size_fallback,
        )?;
        let llvm_options = solc_input.settings.llvm_options.clone();

        let metadata_hash_type = solc_input.settings.metadata.bytecode_hash;
        let append_cbor = solc_input.settings.metadata.append_cbor;

        let mut profiler = solx_codegen_evm::Profiler::default();
        let (mut solc_output, project) = match language {
            solx_standard_json::InputLanguage::Solidity => {
                let run_solc_standard_json =
                    profiler.start_pipeline_element("solc_Solidity_Standard_JSON");
                let mut solc_output = frontend.standard_json(
                    &mut solc_input,
                    use_import_callback,
                    base_path.as_deref(),
                    include_paths.as_slice(),
                    allow_paths,
                )?;
                run_solc_standard_json.borrow_mut().finish();

                solc_input.resolve_sources()?;
                let function_definitions = solc_output.get_debug_info(&solc_input.sources);

                if solc_output.has_errors() {
                    solc_output.write_and_exit(&solc_input.settings.output_selection);
                }
                messages
                    .lock()
                    .expect("lock is never poisoned because worker threads do not panic")
                    .extend(solc_output.errors.drain(..));

                let run_solx_project = profiler.start_pipeline_element("solx_Solidity_IR_Analysis");
                let project = Project::try_from_solidity_output(
                    frontend.version(),
                    solc_input.settings.libraries.clone(),
                    via_ir,
                    &mut solc_output,
                    Some(function_definitions),
                    output_config.as_ref(),
                )?;
                run_solx_project.borrow_mut().finish();
                if solc_output.has_errors() {
                    solc_output.write_and_exit(&solc_input.settings.output_selection);
                }

                (solc_output, project)
            }
            solx_standard_json::InputLanguage::Yul => {
                if solc_input
                    .settings
                    .output_selection
                    .is_debug_info_set_for_any()
                {
                    anyhow::bail!(solx_standard_json::OutputError::new_error(
                        "Debug info is only supported for Solidity source code input."
                    ));
                }

                let run_solc_validate_yul = profiler.start_pipeline_element("solc_Yul_Validation");
                let mut solc_output = frontend.validate_yul_standard_json(&mut solc_input)?;
                run_solc_validate_yul.borrow_mut().finish();
                if solc_output.has_errors() {
                    solc_output.write_and_exit(&solc_input.settings.output_selection);
                }

                let run_solx_yul_project = profiler.start_pipeline_element("solx_Yul_IR_Analysis");
                let project = Project::try_from_yul_sources(
                    frontend.version(),
                    solc_input.sources,
                    solc_input.settings.libraries.clone(),
                    &solc_input.settings.output_selection,
                    Some(&mut solc_output),
                    output_config.as_ref(),
                )?;
                run_solx_yul_project.borrow_mut().finish();
                if solc_output.has_errors() {
                    solc_output.write_and_exit(&solc_input.settings.output_selection);
                }

                (solc_output, project)
            }
            solx_standard_json::InputLanguage::LLVMIR => {
                if solc_input
                    .settings
                    .output_selection
                    .is_debug_info_set_for_any()
                {
                    anyhow::bail!(solx_standard_json::OutputError::new_error(
                        "Debug info is only supported for Solidity source code input."
                    ));
                }

                let mut solc_output = solx_standard_json::Output::new(&solc_input.sources);

                let run_solx_llvm_ir_project =
                    profiler.start_pipeline_element("solx_LLVM_IR_Analysis");
                let project = Project::try_from_llvm_ir_sources(
                    solc_input.sources,
                    solc_input.settings.libraries.clone(),
                    &solc_input.settings.output_selection,
                    Some(&mut solc_output),
                )?;
                run_solx_llvm_ir_project.borrow_mut().finish();
                if solc_output.has_errors() {
                    solc_output.write_and_exit(&solc_input.settings.output_selection);
                }

                (solc_output, project)
            }
        };

        let run_solx_compile = profiler.start_pipeline_element("solx_Compilation");
        let build = project.compile_to_evm(
            messages,
            &solc_input.settings.output_selection,
            solc_input.settings.evm_version,
            metadata_hash_type,
            append_cbor,
            optimizer_settings.clone(),
            llvm_options,
            output_config.clone(),
        )?;
        run_solx_compile.borrow_mut().finish();
        let output_selection = solc_input.settings.output_selection.clone();
        if build.has_errors() {
            build.write_to_standard_json(
                &mut solc_output,
                &solc_input.settings.output_selection,
                false,
                profiler.to_vec(),
            )?;
            solc_output.write_and_exit(&solc_input.settings.output_selection);
        }
        let build = if output_selection.is_bytecode_set_for_any() {
            let run_solx_link = profiler.start_pipeline_element("solx_Linking");
            let build = build.link(linker_symbols);
            run_solx_link.borrow_mut().finish();
            build
        } else {
            build
        };
        build.write_to_standard_json(
            &mut solc_output,
            &output_selection,
            true,
            profiler.to_vec(),
        )?;
        solc_output.write_and_exit(&output_selection);
    }

    ///
    /// Prints the compiler version information to stdout.
    ///
    pub fn print_version<F>(&self, frontend: &F) -> anyhow::Result<()>
    where
        F: Frontend,
    {
        writeln!(
            std::io::stdout(),
            "{DEFAULT_EXECUTABLE_NAME} v{}, {DEFAULT_PACKAGE_DESCRIPTION}, Front end: {}, LLVM build: {}",
            Self::version(),
            frontend.name(),
            inkwell::support::get_commit_id().to_string(),
        )?;
        writeln!(std::io::stdout(), "Version: {}", frontend.version().long)?;
        Ok(())
    }

    /// Returns the compiler version string from the package metadata.
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
