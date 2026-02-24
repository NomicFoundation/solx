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
    /// - the full contract path for all other IR types
    ///
    pub fn identifier(&self) -> &str {
        match self.ir.as_ref() {
            Some(IR::Yul(yul)) => yul.object.identifier.as_str(),
            _ => self.name.full_path.as_str(),
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
            (IR::Yul(mut yul), code_segment) => {
                let (
                    selector_debug_info,
                    selector_llvm_ir_unoptimized,
                    selector_llvm_ir,
                    selector_llvm_assembly,
                ) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (
                        solx_standard_json::InputSelector::BytecodeDebugInfo,
                        solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                        solx_standard_json::InputSelector::BytecodeLLVMIR,
                        solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                    ),
                    solx_utils::CodeSegment::Runtime => (
                        solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                    ),
                };

                let output_debug_info = language == solx_standard_json::InputLanguage::Solidity
                    && output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        selector_debug_info,
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

                let code_identifier = yul.object.identifier.clone();
                let module_name = match code_segment {
                    solx_utils::CodeSegment::Deploy => contract_name.full_path.to_owned(),
                    solx_utils::CodeSegment::Runtime => {
                        format!("{}.{code_segment}", contract_name.full_path)
                    }
                };

                let llvm = inkwell::context::Context::create();
                let module = llvm.create_module(module_name.as_str());
                let mut context = solx_codegen_evm::Context::new(
                    &llvm,
                    module,
                    llvm_options,
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer,
                    output_debug_info,
                    solidity_data,
                    output_config,
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                context.set_yul_data(solx_codegen_evm::ContextYulData::new(identifier_paths));
                let run_yul_lowering = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "YulToLLVMIR",
                    &optimizer_settings,
                );
                yul.object.declare(&mut context)?;
                yul.object.into_llvm(&mut context).map_err(|error| {
                    anyhow::anyhow!("{code_segment} code LLVM IR generator: {error}")
                })?;
                run_yul_lowering.borrow_mut().finish();
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir_unoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir,
                ) {
                    context.set_capture_llvm_ir(true);
                }
                let mut build = context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        selector_llvm_assembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let (immutables_out, metadata_out) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (None, None),
                    solx_utils::CodeSegment::Runtime => (
                        Some(build.immutables.take().unwrap_or_default()),
                        metadata_bytes,
                    ),
                };
                let object = EVMContractObject::from_build(
                    code_identifier,
                    contract_name.clone(),
                    build,
                    true,
                    code_segment,
                    immutables_out,
                    metadata_out,
                    yul.dependencies,
                    profiler.to_vec(),
                );
                Ok(object)
            }
            (IR::EVMLegacyAssembly(mut code), code_segment) => {
                let (
                    selector_debug_info,
                    selector_llvm_ir_unoptimized,
                    selector_llvm_ir,
                    selector_llvm_assembly,
                ) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (
                        solx_standard_json::InputSelector::BytecodeDebugInfo,
                        solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                        solx_standard_json::InputSelector::BytecodeLLVMIR,
                        solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                    ),
                    solx_utils::CodeSegment::Runtime => (
                        solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
                        solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                    ),
                };

                let output_debug_info = language == solx_standard_json::InputLanguage::Solidity
                    && output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        selector_debug_info,
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

                let code_identifier = match code_segment {
                    solx_utils::CodeSegment::Deploy => contract_name.full_path.to_owned(),
                    solx_utils::CodeSegment::Runtime => {
                        format!("{}.{code_segment}", contract_name.full_path)
                    }
                };
                let evmla_data = solx_codegen_evm::ContextEVMLAData::new(
                    solc_version.expect("Always exists").default,
                );

                // Deploy: accumulate dependencies from assembly before it is consumed
                let mut accumulated_dependencies =
                    solx_codegen_evm::Dependencies::new(code_identifier.as_str());
                if matches!(code_segment, solx_utils::CodeSegment::Deploy) {
                    code.assembly
                        .accumulate_evm_dependencies(&mut accumulated_dependencies);
                }

                let llvm = inkwell::context::Context::create();
                let module = llvm.create_module(code_identifier.as_str());
                let mut context = solx_codegen_evm::Context::new(
                    &llvm,
                    module,
                    llvm_options,
                    contract_name.clone(),
                    code_segment,
                    evm_version,
                    optimizer,
                    output_debug_info,
                    solidity_data,
                    output_config,
                );
                inkwell::support::error_handling::install_stack_error_handler(
                    crate::process::evm_stack_error_handler,
                );
                context.set_evmla_data(evmla_data);
                let run_evm_assembly_lowering = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "EVMAssemblyToLLVMIR",
                    &optimizer_settings,
                );
                code.assembly.declare(&mut context)?;
                code.assembly.into_llvm(&mut context).map_err(|error| {
                    anyhow::anyhow!("{code_segment} code LLVM IR generator: {error}")
                })?;
                run_evm_assembly_lowering.borrow_mut().finish();
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir_unoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir,
                ) {
                    context.set_capture_llvm_ir(true);
                }
                let mut build = context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        selector_llvm_assembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let dependencies = match code_segment {
                    solx_utils::CodeSegment::Deploy => accumulated_dependencies,
                    solx_utils::CodeSegment::Runtime => code.dependencies,
                };
                let (immutables_out, metadata_out) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (None, None),
                    solx_utils::CodeSegment::Runtime => (
                        Some(build.immutables.take().unwrap_or_default()),
                        metadata_bytes,
                    ),
                };
                let object = EVMContractObject::from_build(
                    code_identifier,
                    contract_name.clone(),
                    build,
                    false,
                    code_segment,
                    immutables_out,
                    metadata_out,
                    dependencies,
                    profiler.to_vec(),
                );
                Ok(object)
            }
            (IR::LLVMIR(llvm_ir), code_segment) => {
                let code_identifier = match code_segment {
                    solx_utils::CodeSegment::Deploy => contract_name.full_path.to_owned(),
                    solx_utils::CodeSegment::Runtime => {
                        format!("{}.{code_segment}", contract_name.full_path)
                    }
                };
                let memory_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                    &llvm_ir.source.as_bytes()[..llvm_ir.source.len() - 1],
                    code_identifier.as_str(),
                    true,
                );

                let llvm = inkwell::context::Context::create();
                let module = llvm
                    .create_module_from_ir(memory_buffer)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;

                let (selector_llvm_ir_unoptimized, selector_llvm_ir, selector_llvm_assembly) =
                    match code_segment {
                        solx_utils::CodeSegment::Deploy => (
                            solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                            solx_standard_json::InputSelector::BytecodeLLVMIR,
                            solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                        ),
                        solx_utils::CodeSegment::Runtime => (
                            solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                            solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
                            solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                        ),
                    };

                let mut context = solx_codegen_evm::Context::new(
                    &llvm,
                    module,
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
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir_unoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir,
                ) {
                    context.set_capture_llvm_ir(true);
                }
                let build = context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        selector_llvm_assembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let (immutables_out, metadata_out) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (None, None),
                    solx_utils::CodeSegment::Runtime => (Some(BTreeMap::new()), metadata_bytes),
                };
                let object = EVMContractObject::from_build(
                    code_identifier,
                    contract_name.clone(),
                    build,
                    false,
                    code_segment,
                    immutables_out,
                    metadata_out,
                    llvm_ir.dependencies,
                    profiler.to_vec(),
                );
                Ok(object)
            }
            #[cfg(feature = "mlir")]
            (IR::MLIR(mlir), code_segment) => {
                let code_identifier = match code_segment {
                    solx_utils::CodeSegment::Deploy => contract_name.full_path.to_owned(),
                    solx_utils::CodeSegment::Runtime => {
                        format!("{}.{code_segment}", contract_name.full_path)
                    }
                };

                let mlir_context = solx_mlir::Context::new();
                let llvm_module = mlir_context
                    .try_into_llvm_module_from_source(&mlir.source)
                    .map_err(|error| anyhow::anyhow!("MLIR translation: {error}"))?;

                let (raw_module, raw_context) = llvm_module.into_raw();
                let llvm = unsafe { inkwell::context::Context::new(raw_context) };
                let module = unsafe { inkwell::module::Module::new(raw_module) };

                let (selector_llvm_ir_unoptimized, selector_llvm_ir, selector_llvm_assembly) =
                    match code_segment {
                        solx_utils::CodeSegment::Deploy => (
                            solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                            solx_standard_json::InputSelector::BytecodeLLVMIR,
                            solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                        ),
                        solx_utils::CodeSegment::Runtime => (
                            solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                            solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
                            solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                        ),
                    };

                let mut context = solx_codegen_evm::Context::new(
                    &llvm,
                    module,
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
                if output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir_unoptimized,
                ) || output_selection.check_selection(
                    contract_name.path.as_str(),
                    contract_name.name.as_deref(),
                    selector_llvm_ir,
                ) {
                    context.set_capture_llvm_ir(true);
                }
                let build = context.build(
                    output_selection.check_selection(
                        contract_name.path.as_str(),
                        contract_name.name.as_deref(),
                        selector_llvm_assembly,
                    ),
                    output_bytecode,
                    false,
                    &mut profiler,
                )?;
                let (immutables_out, metadata_out) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (None, None),
                    solx_utils::CodeSegment::Runtime => (Some(BTreeMap::new()), metadata_bytes),
                };
                let object = EVMContractObject::from_build(
                    code_identifier.clone(),
                    contract_name.clone(),
                    build,
                    false,
                    code_segment,
                    immutables_out,
                    metadata_out,
                    solx_codegen_evm::Dependencies::new(code_identifier.as_str()),
                    profiler.to_vec(),
                );
                Ok(object)
            }
        }
    }
}
