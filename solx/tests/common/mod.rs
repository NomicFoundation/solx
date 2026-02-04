//!
//! Unit test common utilities.
//!

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod r#const;

pub use self::r#const::*;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Once;

use assert_cmd::Command;

use solx_core::Solc;
use solx_standard_json::CollectableError;

/// Shared lock for unit tests, as `solc` libraries are not thread-safe.
pub static UNIT_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

///
/// Setup required test dependencies.
///
pub fn setup() -> anyhow::Result<()> {
    // Set the `solx` binary path
    let solx_bin = Command::new(assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")));
    let _ = solx_core::process::EXECUTABLE.set(PathBuf::from(solx_bin.get_program()));

    // Enable LLVM pretty stack trace
    inkwell::support::enable_llvm_pretty_stack_trace();

    Ok(())
}

///
/// Reads source code files from the disk.
///
pub fn read_sources(paths: &[&str]) -> BTreeMap<String, String> {
    paths
        .iter()
        .map(|path| {
            let result = std::fs::read_to_string(path).map_err(|error| anyhow::anyhow!(error));
            result.map(|result| ((*path).to_owned(), result))
        })
        .collect::<anyhow::Result<BTreeMap<String, String>>>()
        .expect("Source reading failure")
}

///
/// Builds the Solidity project and returns the standard JSON output.
///
pub fn build_solidity_standard_json(
    sources: BTreeMap<String, String>,
    libraries: solx_utils::Libraries,
    metadata_hash_type: solx_utils::MetadataHashType,
    remappings: BTreeSet<String>,
    via_ir: bool,
    optimizer_settings: solx_codegen_evm::OptimizerSettings,
) -> anyhow::Result<solx_standard_json::Output> {
    self::setup()?;

    let solc_compiler = solx::Solc::default();

    solx_codegen_evm::initialize_target();

    let sources: BTreeMap<String, solx_standard_json::InputSource> = sources
        .into_iter()
        .map(|(path, source)| (path, solx_standard_json::InputSource::from(source)))
        .collect();

    let mut selectors = BTreeSet::new();
    selectors.insert(solx_standard_json::InputSelector::BytecodeObject);
    selectors.insert(solx_standard_json::InputSelector::BytecodeLinkReferences);
    selectors.insert(solx_standard_json::InputSelector::BytecodeOpcodes);
    selectors.insert(solx_standard_json::InputSelector::BytecodeLLVMAssembly);
    selectors.insert(solx_standard_json::InputSelector::BytecodeSourceMap);
    selectors.insert(solx_standard_json::InputSelector::BytecodeFunctionDebugData);
    selectors.insert(solx_standard_json::InputSelector::BytecodeGeneratedSources);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeObject);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeLinkReferences);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeImmutableReferences);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeOpcodes);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeSourceMap);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeFunctionDebugData);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeGeneratedSources);
    selectors.insert(solx_standard_json::InputSelector::GasEstimates);
    selectors.insert(solx_standard_json::InputSelector::AST);
    selectors.insert(solx_standard_json::InputSelector::ABI);
    selectors.insert(solx_standard_json::InputSelector::Metadata);
    selectors.insert(solx_standard_json::InputSelector::DeveloperDocumentation);
    selectors.insert(solx_standard_json::InputSelector::UserDocumentation);
    selectors.insert(solx_standard_json::InputSelector::StorageLayout);
    selectors.insert(solx_standard_json::InputSelector::TransientStorageLayout);
    selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
    selectors.insert(if via_ir {
        solx_standard_json::InputSelector::Yul
    } else {
        solx_standard_json::InputSelector::EVMLegacyAssembly
    });
    selectors.insert(solx_standard_json::InputSelector::Benchmarks);
    let output_selection = solx_standard_json::InputSelection::new(selectors);

    let mut input = solx_standard_json::Input::try_from_solidity_sources(
        sources,
        libraries.clone(),
        remappings,
        solx_standard_json::InputOptimizer::default(),
        None,
        via_ir,
        &output_selection,
        solx_standard_json::InputMetadata::default(),
        vec![],
    )?;

    let mut output = {
        let _lock = UNIT_TEST_LOCK.lock();
        solc_compiler.standard_json(&mut input, true, None, &[], None)
    }?;
    output.check_errors()?;

    let linker_symbols = libraries.as_linker_symbols()?;

    let project = solx_core::Project::try_from_solc_output(
        solc_compiler.version(),
        libraries,
        via_ir,
        &mut output,
        None,
        None,
    )?;
    output.check_errors()?;

    let build = project.compile_to_evm(
        Arc::new(Mutex::new(vec![])),
        &input.settings.output_selection,
        input.settings.evm_version,
        metadata_hash_type,
        input.settings.metadata.append_cbor,
        optimizer_settings,
        vec![],
        None,
    )?;
    build.check_errors()?;

    let build = if input.settings.output_selection.is_bytecode_set_for_any() {
        build.link(linker_symbols)
    } else {
        build
    };
    build.write_to_standard_json(&mut output, &input.settings.output_selection, true, vec![])?;
    output.check_errors()?;
    Ok(output)
}

///
/// Builds the Yul standard JSON and returns the standard JSON output.
///
/// If `solc_compiler` is set, the standard JSON is validated with `solc`.
///
pub fn build_yul_standard_json(
    mut input: solx_standard_json::Input,
) -> anyhow::Result<solx_standard_json::Output> {
    self::setup()?;

    let solc_compiler = solx::Solc::default();

    solx_codegen_evm::initialize_target();

    let optimizer_settings = solx_codegen_evm::OptimizerSettings::try_from_cli(
        input.settings.optimizer.mode.unwrap_or_else(|| {
            solx_standard_json::InputOptimizer::default_mode().expect("Always exists")
        }),
    )?;

    let mut solc_output = {
        let _lock = UNIT_TEST_LOCK.lock();
        solc_compiler.validate_yul_standard_json(&mut input)
    }?;

    let project = solx_core::Project::try_from_yul_sources(
        solc_compiler.version(),
        input.sources,
        solx_utils::Libraries::default(),
        &input.settings.output_selection,
        Some(&mut solc_output),
        None,
    )?;
    let build = project.compile_to_evm(
        Arc::new(Mutex::new(vec![])),
        &input.settings.output_selection,
        input.settings.evm_version,
        solx_utils::MetadataHashType::IPFS,
        input.settings.metadata.append_cbor,
        optimizer_settings,
        vec![],
        None,
    )?;
    build.check_errors()?;

    let build = if input.settings.output_selection.is_bytecode_set_for_any() {
        build.link(BTreeMap::new())
    } else {
        build
    };
    build.write_to_standard_json(
        &mut solc_output,
        &input.settings.output_selection,
        true,
        vec![],
    )?;
    solc_output.check_errors()?;
    Ok(solc_output)
}

///
/// Builds the Solidity project with debug info enabled and returns the standard JSON output.
/// The `via_ir` parameter controls whether to use Yul (true) or EVM assembly (false) pipeline.
///
pub fn build_solidity_standard_json_debug_info_with_via_ir(
    sources: BTreeMap<String, String>,
    via_ir: bool,
) -> anyhow::Result<solx_standard_json::Output> {
    self::setup()?;

    let solc_compiler = solx::Solc::default();

    solx_codegen_evm::initialize_target();

    let sources: BTreeMap<String, solx_standard_json::InputSource> = sources
        .into_iter()
        .map(|(path, source)| (path, solx_standard_json::InputSource::from(source)))
        .collect();

    let mut selectors = BTreeSet::new();
    selectors.insert(solx_standard_json::InputSelector::BytecodeObject);
    selectors.insert(solx_standard_json::InputSelector::BytecodeDebugInfo);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeObject);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo);
    let output_selection = solx_standard_json::InputSelection::new(selectors);

    let mut input = solx_standard_json::Input::try_from_solidity_sources(
        sources,
        solx_utils::Libraries::default(),
        BTreeSet::new(),
        solx_standard_json::InputOptimizer::default(),
        None,
        via_ir,
        &output_selection,
        solx_standard_json::InputMetadata::default(),
        vec![],
    )?;

    let mut output = {
        let _lock = UNIT_TEST_LOCK.lock();
        solc_compiler.standard_json(&mut input, true, None, &[], None)
    }?;
    output.check_errors()?;

    input.resolve_sources()?;
    let debug_info = output.get_debug_info(&input.sources);

    let project = solx_core::Project::try_from_solc_output(
        solc_compiler.version(),
        solx_utils::Libraries::default(),
        via_ir,
        &mut output,
        Some(debug_info),
        None,
    )?;
    output.check_errors()?;

    let build = project.compile_to_evm(
        Arc::new(Mutex::new(vec![])),
        &input.settings.output_selection,
        input.settings.evm_version,
        solx_utils::MetadataHashType::None,
        false, // no CBOR metadata for cleaner bytecode
        solx_codegen_evm::OptimizerSettings::cycles(),
        vec![],
        None,
    )?;
    build.check_errors()?;

    let build = if input.settings.output_selection.is_bytecode_set_for_any() {
        build.link(BTreeMap::new())
    } else {
        build
    };
    build.write_to_standard_json(&mut output, &input.settings.output_selection, true, vec![])?;
    output.check_errors()?;
    Ok(output)
}

///
/// Builds the LLVM IR standard JSON and returns the standard JSON output.
///
pub fn build_llvm_ir_standard_json(
    input: solx_standard_json::Input,
) -> anyhow::Result<solx_standard_json::Output> {
    self::setup()?;

    solx_codegen_evm::initialize_target();

    let optimizer_settings = solx_codegen_evm::OptimizerSettings::try_from_cli(
        input.settings.optimizer.mode.unwrap_or_else(|| {
            solx_standard_json::InputOptimizer::default_mode().expect("Always exists")
        }),
    )?;

    let mut output = solx_standard_json::Output::new(&BTreeMap::new());

    let project = solx_core::Project::try_from_llvm_ir_sources(
        input.sources,
        input.settings.libraries,
        &input.settings.output_selection,
        Some(&mut output),
    )?;
    let build = project.compile_to_evm(
        Arc::new(Mutex::new(vec![])),
        &input.settings.output_selection,
        input.settings.evm_version,
        solx_utils::MetadataHashType::IPFS,
        input.settings.metadata.append_cbor,
        optimizer_settings,
        vec![],
        None,
    )?;
    build.check_errors()?;

    let build = if input.settings.output_selection.is_bytecode_set_for_any() {
        build.link(BTreeMap::new())
    } else {
        build
    };
    build.write_to_standard_json(&mut output, &input.settings.output_selection, true, vec![])?;
    output.check_errors()?;
    Ok(output)
}
