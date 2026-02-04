//!
//! Contract data.
//!

pub mod ir;
pub mod metadata;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use solx_codegen_evm::IContext;

use crate::build::contract::object::Object as EVMContractObject;
use crate::error::Error;

use self::ir::IR;

///
/// Contract data.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Contract {
    /// Contract name.
    pub name: solx_utils::ContractName,
    /// IR source code data.
    pub ir: Option<IR>,
    /// solc metadata.
    pub metadata: Option<String>,
    /// solc ABI.
    pub abi: Option<serde_json::Value>,
    /// solc method identifiers.
    pub method_identifiers: Option<BTreeMap<String, String>>,
    /// solc user documentation.
    pub userdoc: Option<serde_json::Value>,
    /// solc developer documentation.
    pub devdoc: Option<serde_json::Value>,
    /// solc storage layout.
    pub storage_layout: Option<serde_json::Value>,
    /// solc transient storage layout.
    pub transient_storage_layout: Option<serde_json::Value>,
    /// solc EVM legacy assembly.
    pub legacy_assembly: Option<solx_evm_assembly::Assembly>,
    /// solc Yul IR.
    pub yul: Option<String>,
}

impl Contract {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        name: solx_utils::ContractName,
        ir: Option<IR>,
        metadata: Option<String>,
        abi: Option<serde_json::Value>,
        method_identifiers: Option<BTreeMap<String, String>>,
        userdoc: Option<serde_json::Value>,
        devdoc: Option<serde_json::Value>,
        storage_layout: Option<serde_json::Value>,
        transient_storage_layout: Option<serde_json::Value>,
        legacy_assembly: Option<solx_evm_assembly::Assembly>,
        yul: Option<String>,
    ) -> Self {
        Self {
            name,
            ir,
            metadata,
            abi,
            method_identifiers,
            userdoc,
            devdoc,
            storage_layout,
            transient_storage_layout,
            legacy_assembly,
            yul,
        }
    }

    ///
    /// Returns the contract identifier, which is:
    /// - the Yul object identifier for Yul
    /// - the full contract path for EVM legacy assembly
    /// - the module name for LLVM IR
    ///
    pub fn identifier(&self) -> &str {
        match self.ir.as_ref() {
            Some(IR::Yul(yul)) => yul.object.identifier.as_str(),
            Some(IR::EVMLegacyAssembly(evm)) => evm.assembly.full_path(),
            Some(IR::LLVMIR(llvm_ir)) => llvm_ir.path.as_str(),
            None => self.name.full_path.as_str(),
        }
    }

    ///
    /// Compiles the specified contract to EVM, returning its build artifacts.
    ///
    pub fn compile_to_evm(
        language: solx_standard_json::InputLanguage,
        solc_version: Option<solx_standard_json::Version>,
        contract_name: solx_utils::ContractName,
        contract_ir: IR,
        code_segment: solx_utils::CodeSegment,
        evm_version: Option<solx_utils::EVMVersion>,
        identifier_paths: BTreeMap<String, String>,
        debug_info: Option<solx_utils::DebugInfo>,
        output_selection: solx_standard_json::InputSelection,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        metadata_bytes: Option<Vec<u8>>,
        mut optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> Result<EVMContractObject, Error> {
        use solx_codegen_evm::WriteLLVM;
        let mut profiler = solx_codegen_evm::Profiler::default();

        if let Some(metadata_bytes) = metadata_bytes.as_ref() {
            optimizer_settings.set_metadata_size(metadata_bytes.len() as u64);
        }
        let optimizer = solx_codegen_evm::Optimizer::new(optimizer_settings.clone());

        let output_bytecode = output_selection.is_bytecode_set_for_any();
        match (contract_ir, code_segment) {
            (IR::Yul(mut yul), solx_utils::CodeSegment::Deploy) => {
                let output_debug_info = language == solx_standard_json::InputLanguage::Solidity
                    && output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::BytecodeDebugInfo,
                    );
                let solidity_data = if language == solx_standard_json::InputLanguage::Solidity {
                    Some(solx_codegen_evm::ContextSolidityData::new(
                        immutables,
                        yul.object.sources.clone(),
                        debug_info,
                    ))
                } else {
                    None
                };

                let deploy_code_identifier = yul.object.identifier.clone();

                let deploy_llvm = inkwell::context::Context::create();
                let deploy_module = deploy_llvm.create_module(contract_name.full_path.as_str());
                let mut deploy_context = solx_codegen_evm::Context::new(
                    &deploy_llvm,
                    deploy_module,
                    llvm_options.clone(),
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer,
                    output_debug_info,
                    solidity_data,
                    output_config.clone(),
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                deploy_context
                    .set_yul_data(solx_codegen_evm::ContextYulData::new(identifier_paths));
                let run_yul_lowering = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "YulToLLVMIR",
                    &optimizer_settings,
                );
                yul.object.declare(&mut deploy_context)?;
                yul.object.into_llvm(&mut deploy_context).map_err(|error| {
                    anyhow::anyhow!("{code_segment} code LLVM IR generator: {error}")
                })?;
                run_yul_lowering.borrow_mut().finish();
                // Enable LLVM IR capture for standard JSON output
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::BytecodeLLVMIROptimized,
                ) {
                    deploy_context.set_capture_llvm_ir(true);
                }
                let deploy_build = deploy_context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let deploy_object = EVMContractObject::new(
                    deploy_code_identifier,
                    contract_name.clone(),
                    deploy_build.assembly,
                    deploy_build.bytecode,
                    deploy_build.debug_info,
                    deploy_build.evmla,
                    deploy_build.ethir,
                    deploy_build.llvm_ir_unoptimized,
                    deploy_build.llvm_ir_optimized,
                    true,
                    code_segment,
                    None,
                    None,
                    yul.dependencies,
                    deploy_build.is_size_fallback,
                    deploy_build.warnings,
                    profiler.to_vec(),
                );
                Ok(deploy_object)
            }
            (IR::Yul(mut yul), solx_utils::CodeSegment::Runtime) => {
                let output_debug_info = language == solx_standard_json::InputLanguage::Solidity
                    && output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                    );
                let solidity_data = if language == solx_standard_json::InputLanguage::Solidity {
                    Some(solx_codegen_evm::ContextSolidityData::new(
                        immutables,
                        yul.object.sources.clone(),
                        debug_info,
                    ))
                } else {
                    None
                };

                let runtime_code_identifier = yul.object.identifier.clone();

                let runtime_llvm = inkwell::context::Context::create();
                let runtime_module = runtime_llvm
                    .create_module(format!("{}.{code_segment}", contract_name.full_path).as_str());
                let mut runtime_context = solx_codegen_evm::Context::new(
                    &runtime_llvm,
                    runtime_module,
                    llvm_options.clone(),
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer.clone(),
                    output_debug_info,
                    solidity_data,
                    output_config.clone(),
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                runtime_context.set_yul_data(solx_codegen_evm::ContextYulData::new(
                    identifier_paths.clone(),
                ));
                let run_yul_lowering = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "YulToLLVMIR",
                    &optimizer_settings,
                );
                yul.object.declare(&mut runtime_context)?;
                yul.object
                    .into_llvm(&mut runtime_context)
                    .map_err(|error| {
                        anyhow::anyhow!("{code_segment} code LLVM IR generator: {error}")
                    })?;
                run_yul_lowering.borrow_mut().finish();
                // Enable LLVM IR capture for standard JSON output
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::RuntimeBytecodeLLVMIROptimized,
                ) {
                    runtime_context.set_capture_llvm_ir(true);
                }
                let runtime_build = runtime_context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let immutables = runtime_build.immutables.unwrap_or_default();
                let runtime_object = EVMContractObject::new(
                    runtime_code_identifier,
                    contract_name.clone(),
                    runtime_build.assembly,
                    runtime_build.bytecode,
                    runtime_build.debug_info,
                    runtime_build.evmla,
                    runtime_build.ethir,
                    runtime_build.llvm_ir_unoptimized,
                    runtime_build.llvm_ir_optimized,
                    true,
                    code_segment,
                    Some(immutables),
                    metadata_bytes,
                    yul.dependencies,
                    runtime_build.is_size_fallback,
                    runtime_build.warnings,
                    profiler.to_vec(),
                );
                Ok(runtime_object)
            }
            (IR::EVMLegacyAssembly(mut deploy_code), solx_utils::CodeSegment::Deploy) => {
                let output_debug_info = language == solx_standard_json::InputLanguage::Solidity
                    && output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::BytecodeDebugInfo,
                    );
                let source_ids = debug_info
                    .as_ref()
                    .map(|info| info.source_ids.clone())
                    .unwrap_or_default();
                let solidity_data = if language == solx_standard_json::InputLanguage::Solidity {
                    Some(solx_codegen_evm::ContextSolidityData::new(
                        immutables, source_ids, debug_info,
                    ))
                } else {
                    None
                };

                let evmla_data = solx_codegen_evm::ContextEVMLAData::new(
                    solc_version.expect("Always exists").default,
                );
                let deploy_code_identifier = contract_name.full_path.to_owned();
                let mut deploy_code_dependencies =
                    solx_codegen_evm::Dependencies::new(deploy_code_identifier.as_str());
                deploy_code
                    .assembly
                    .accumulate_evm_dependencies(&mut deploy_code_dependencies);

                let deploy_llvm = inkwell::context::Context::create();
                let deploy_module = deploy_llvm.create_module(deploy_code_identifier.as_str());
                let mut deploy_context = solx_codegen_evm::Context::new(
                    &deploy_llvm,
                    deploy_module,
                    llvm_options.clone(),
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer,
                    output_debug_info,
                    solidity_data,
                    output_config.clone(),
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                deploy_context.set_evmla_data(evmla_data);
                let run_evm_assembly_lowering = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "EVMAssemblyToLLVMIR",
                    &optimizer_settings,
                );
                deploy_code.assembly.declare(&mut deploy_context)?;
                deploy_code
                    .assembly
                    .into_llvm(&mut deploy_context)
                    .map_err(|error| {
                        anyhow::anyhow!("{code_segment} code LLVM IR generator: {error}")
                    })?;
                run_evm_assembly_lowering.borrow_mut().finish();
                // Enable LLVM IR capture for standard JSON output
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::BytecodeLLVMIROptimized,
                ) {
                    deploy_context.set_capture_llvm_ir(true);
                }
                let deploy_build = deploy_context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let deploy_object = EVMContractObject::new(
                    deploy_code_identifier,
                    contract_name.clone(),
                    deploy_build.assembly,
                    deploy_build.bytecode,
                    deploy_build.debug_info,
                    deploy_build.evmla,
                    deploy_build.ethir,
                    deploy_build.llvm_ir_unoptimized,
                    deploy_build.llvm_ir_optimized,
                    false,
                    code_segment,
                    None,
                    None,
                    deploy_code_dependencies,
                    deploy_build.is_size_fallback,
                    deploy_build.warnings,
                    profiler.to_vec(),
                );
                Ok(deploy_object)
            }
            (IR::EVMLegacyAssembly(mut runtime_code), solx_utils::CodeSegment::Runtime) => {
                let output_debug_info = language == solx_standard_json::InputLanguage::Solidity
                    && output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                    );
                let source_ids = debug_info
                    .as_ref()
                    .map(|info| info.source_ids.clone())
                    .unwrap_or_default();
                let solidity_data = if language == solx_standard_json::InputLanguage::Solidity {
                    Some(solx_codegen_evm::ContextSolidityData::new(
                        immutables, source_ids, debug_info,
                    ))
                } else {
                    None
                };

                let runtime_code_identifier = format!("{}.{code_segment}", contract_name.full_path);
                let evmla_data = solx_codegen_evm::ContextEVMLAData::new(
                    solc_version.expect("Always exists").default,
                );

                let runtime_llvm = inkwell::context::Context::create();
                let runtime_module = runtime_llvm.create_module(runtime_code_identifier.as_str());
                let mut runtime_context = solx_codegen_evm::Context::new(
                    &runtime_llvm,
                    runtime_module,
                    llvm_options.clone(),
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer.clone(),
                    output_debug_info,
                    solidity_data,
                    output_config.clone(),
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                runtime_context.set_evmla_data(evmla_data.clone());
                let run_evm_assembly_lowering = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "EVMAssemblyToLLVMIR",
                    &optimizer_settings,
                );
                runtime_code.assembly.declare(&mut runtime_context)?;
                runtime_code
                    .assembly
                    .into_llvm(&mut runtime_context)
                    .map_err(|error| {
                        anyhow::anyhow!("{code_segment} code LLVM IR generator: {error}")
                    })?;
                run_evm_assembly_lowering.borrow_mut().finish();
                // Enable LLVM IR capture for standard JSON output
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::RuntimeBytecodeLLVMIROptimized,
                ) {
                    runtime_context.set_capture_llvm_ir(true);
                }
                let runtime_build = runtime_context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let immutables = runtime_build.immutables.unwrap_or_default();
                let runtime_object = EVMContractObject::new(
                    runtime_code_identifier,
                    contract_name.clone(),
                    runtime_build.assembly,
                    runtime_build.bytecode,
                    runtime_build.debug_info,
                    runtime_build.evmla,
                    runtime_build.ethir,
                    runtime_build.llvm_ir_unoptimized,
                    runtime_build.llvm_ir_optimized,
                    false,
                    code_segment,
                    Some(immutables),
                    metadata_bytes,
                    runtime_code.dependencies,
                    runtime_build.is_size_fallback,
                    runtime_build.warnings,
                    profiler.to_vec(),
                );
                Ok(runtime_object)
            }
            (IR::LLVMIR(deploy_llvm_ir), solx_utils::CodeSegment::Deploy) => {
                let deploy_code_identifier = contract_name.full_path.to_owned();
                let deploy_memory_buffer =
                    inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                        &deploy_llvm_ir.source.as_bytes()[..deploy_llvm_ir.source.len() - 1],
                        deploy_code_identifier.as_str(),
                        true,
                    );

                let deploy_llvm = inkwell::context::Context::create();
                let deploy_module = deploy_llvm
                    .create_module_from_ir(deploy_memory_buffer)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                let mut deploy_context = solx_codegen_evm::Context::new(
                    &deploy_llvm,
                    deploy_module,
                    llvm_options,
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer,
                    false,
                    None,
                    output_config,
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                // Enable LLVM IR capture for standard JSON output
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::BytecodeLLVMIROptimized,
                ) {
                    deploy_context.set_capture_llvm_ir(true);
                }
                let deploy_build = deploy_context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let deploy_object = EVMContractObject::new(
                    deploy_code_identifier,
                    contract_name.clone(),
                    deploy_build.assembly,
                    deploy_build.bytecode,
                    deploy_build.debug_info,
                    deploy_build.evmla,
                    deploy_build.ethir,
                    deploy_build.llvm_ir_unoptimized,
                    deploy_build.llvm_ir_optimized,
                    false,
                    code_segment,
                    None,
                    None,
                    deploy_llvm_ir.dependencies,
                    deploy_build.is_size_fallback,
                    deploy_build.warnings,
                    profiler.to_vec(),
                );
                Ok(deploy_object)
            }
            (IR::LLVMIR(runtime_llvm_ir), solx_utils::CodeSegment::Runtime) => {
                let runtime_code_identifier = format!("{}.{code_segment}", contract_name.full_path);
                let runtime_memory_buffer =
                    inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                        &runtime_llvm_ir.source.as_bytes()[..runtime_llvm_ir.source.len() - 1],
                        runtime_code_identifier.as_str(),
                        true,
                    );

                let runtime_llvm = inkwell::context::Context::create();
                let runtime_module = runtime_llvm
                    .create_module_from_ir(runtime_memory_buffer)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                let mut runtime_context = solx_codegen_evm::Context::new(
                    &runtime_llvm,
                    runtime_module,
                    llvm_options.clone(),
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer,
                    false,
                    None,
                    output_config.clone(),
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                // Enable LLVM IR capture for standard JSON output
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    solx_standard_json::InputSelector::RuntimeBytecodeLLVMIROptimized,
                ) {
                    runtime_context.set_capture_llvm_ir(true);
                }
                let runtime_build = runtime_context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let runtime_object = EVMContractObject::new(
                    runtime_code_identifier,
                    contract_name.clone(),
                    runtime_build.assembly,
                    runtime_build.bytecode,
                    runtime_build.debug_info,
                    runtime_build.evmla,
                    runtime_build.ethir,
                    runtime_build.llvm_ir_unoptimized,
                    runtime_build.llvm_ir_optimized,
                    false,
                    code_segment,
                    Some(BTreeMap::new()),
                    metadata_bytes,
                    runtime_llvm_ir.dependencies,
                    runtime_build.is_size_fallback,
                    runtime_build.warnings,
                    profiler.to_vec(),
                );
                Ok(runtime_object)
            }
        }
    }
}
