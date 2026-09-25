//!
//! Contract data.
//!

pub mod ir;
pub mod metadata;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use anyhow::Context as _;

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
    /// MLIR pipeline output.
    pub mlir: Option<solx_mlir::MlirOutput>,
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
        mlir: Option<solx_mlir::MlirOutput>,
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
            mlir,
        }
    }

    ///
    /// Returns an estimated compilation cost for LPT scheduling.
    ///
    /// Used to sort contracts by descending cost before parallel dispatch,
    /// so that large contracts begin compiling first.
    ///
    pub fn estimated_compilation_cost(&self) -> usize {
        match &self.ir {
            Some(IR::LLVMIR(llvm_ir)) => llvm_ir.source.len(),
            Some(IR::MLIR(mlir)) => {
                mlir.source.len()
                    + mlir
                        .runtime_code
                        .as_ref()
                        .map_or(0, |runtime| runtime.source.len())
            }
            None => 0,
        }
    }

    ///
    /// Compiles the specified contract to EVM, returning its build artifacts.
    ///
    pub fn compile_to_evm(
        contract_name: solx_utils::ContractName,
        contract_ir: IR,
        code_segment: solx_utils::CodeSegment,
        output_selection: &solx_standard_json::InputSelection,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        metadata_bytes: Option<Vec<u8>>,
        mut optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: Vec<String>,
        output_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> Result<EVMContractObject, Error> {
        let mut profiler = solx_utils::Profiler::default();

        if let Some(metadata_bytes) = metadata_bytes.as_ref() {
            optimizer_settings.set_metadata_size(metadata_bytes.len() as u64);
        }
        let optimizer = solx_codegen_evm::Optimizer::new(optimizer_settings.clone());
        let optimizer_mode = optimizer_settings.to_string();
        let spill_area_size = optimizer_settings.spill_area_size();

        let output_bytecode = output_selection.is_bytecode_set_for_any();
        match (contract_ir, code_segment) {
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
                    code_segment,
                    optimizer,
                    output_config,
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
                    optimizer_settings.is_fallback_to_size_active(),
                    &mut profiler,
                )?;
                let (immutables_out, metadata_out) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (None, None),
                    solx_utils::CodeSegment::Runtime => (Some(BTreeMap::new()), metadata_bytes),
                };
                let object = EVMContractObject::new(
                    code_identifier,
                    contract_name.clone(),
                    build.assembly,
                    build.bytecode,
                    build.llvm_ir_unoptimized,
                    build.llvm_ir,
                    code_segment,
                    immutables_out,
                    metadata_out,
                    llvm_ir.dependencies,
                    build.is_size_fallback,
                    build.warnings,
                    profiler.to_vec(),
                );
                Ok(object)
            }
            (IR::MLIR(mlir), code_segment) => {
                let code_identifier = match code_segment {
                    solx_utils::CodeSegment::Deploy => contract_name.full_path.to_owned(),
                    solx_utils::CodeSegment::Runtime => format!(
                        "{}{}",
                        contract_name.full_path,
                        solx_utils::Dependencies::DEPLOYED_OBJECT_SUFFIX
                    ),
                };

                let run_context_creation = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "CreateMLIRContext",
                    optimizer_mode.as_str(),
                    spill_area_size,
                );
                let melior = solx_mlir::Context::create_melior_context();
                run_context_creation.borrow_mut().finish();
                let immutables = match code_segment {
                    solx_utils::CodeSegment::Deploy => immutables.unwrap_or_else(|| {
                        BTreeMap::from([(
                            solx_codegen_evm::LIBRARY_DEPLOY_ADDRESS_TAG.to_owned(),
                            BTreeSet::from([0]),
                        )])
                    }),
                    solx_utils::CodeSegment::Runtime => BTreeMap::new(),
                };
                let run_mlir_parsing = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "ParseMLIR",
                    optimizer_mode.as_str(),
                    spill_area_size,
                );
                let mlir_module = solx_mlir::Context::parse_source(&melior, &mlir.source)
                    .context("MLIR translation")?;
                run_mlir_parsing.borrow_mut().finish();

                let run_mlir_translation = profiler.start_evm_translation_unit(
                    contract_name.full_path.as_str(),
                    code_segment,
                    "MLIRToLLVMIR",
                    optimizer_mode.as_str(),
                    spill_area_size,
                );
                let raw_llvm =
                    solx_mlir::Context::translate_module_to_llvm(mlir_module, &immutables)
                        .context("MLIR translation")?;
                run_mlir_translation.borrow_mut().finish();
                let context = unsafe { inkwell::context::Context::new(raw_llvm.context) };
                let module = unsafe { inkwell::module::Module::new(raw_llvm.module) };
                module.set_name(code_identifier.as_str());

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
                    &context,
                    module,
                    llvm_options,
                    code_segment,
                    optimizer,
                    output_config,
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
                    optimizer_settings.is_fallback_to_size_active(),
                    &mut profiler,
                )?;
                let (immutables_out, metadata_out) = match code_segment {
                    solx_utils::CodeSegment::Deploy => (None, None),
                    solx_utils::CodeSegment::Runtime => {
                        (Some(build.immutables.unwrap_or_default()), metadata_bytes)
                    }
                };
                let object = EVMContractObject::new(
                    code_identifier,
                    contract_name.clone(),
                    build.assembly,
                    build.bytecode,
                    build.llvm_ir_unoptimized,
                    build.llvm_ir,
                    code_segment,
                    immutables_out,
                    metadata_out,
                    mlir.dependencies,
                    build.is_size_fallback,
                    build.warnings,
                    profiler.to_vec(),
                );
                Ok(object)
            }
        }
    }
}
