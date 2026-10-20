//!
//! The bitcode of a contract's code segments, and the MLIR text captured for output.
//!

///
/// The LLVM bitcode codegen compiles for a single contract, and the MLIR text captured for output.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MlirOutput {
    /// Pre-pass Sol dialect text of the deploy module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deploy_sol_source: Option<String>,
    /// LLVM dialect text of the deploy module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub deploy_source: String,
    /// LLVM bitcode of the deploy module.
    #[serde(skip)]
    pub deploy_bitcode: Vec<u8>,
    /// Objects the deploy code may embed, its runtime child leading.
    pub deploy_dependencies: solx_utils::Dependencies,
    /// Pre-pass Sol dialect text of the runtime module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_sol_source: Option<String>,
    /// LLVM dialect text of the runtime module.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub runtime_source: String,
    /// LLVM bitcode of the runtime module.
    #[serde(skip)]
    pub runtime_bitcode: Vec<u8>,
    /// Objects the runtime code may embed.
    pub runtime_dependencies: solx_utils::Dependencies,
}

impl MlirOutput {
    /// Joins the outputs of a contract's deploy and runtime modules.
    pub fn new(deploy: SegmentOutput, runtime: SegmentOutput) -> Self {
        Self {
            deploy_sol_source: deploy.sol_source,
            deploy_source: deploy.source.unwrap_or_default(),
            deploy_bitcode: deploy.bitcode,
            deploy_dependencies: deploy.dependencies,
            runtime_sol_source: runtime.sol_source,
            runtime_source: runtime.source.unwrap_or_default(),
            runtime_bitcode: runtime.bitcode,
            runtime_dependencies: runtime.dependencies,
        }
    }
}

///
/// The LLVM bitcode of one code segment of a contract, and the MLIR text captured for output.
///
pub struct SegmentOutput {
    /// Pre-pass Sol dialect text.
    pub sol_source: Option<String>,
    /// LLVM dialect text.
    pub source: Option<String>,
    /// LLVM bitcode.
    pub bitcode: Vec<u8>,
    /// Objects the code may embed.
    pub dependencies: solx_utils::Dependencies,
}

impl SegmentOutput {
    /// The optimized deploy code of a contract whose constructor chain does nothing, guarding
    /// against value unless `payable`. With `emit_debug_info`, its locations are the `(line,
    /// column)` of the contract and of its constructor in `source_file`.
    pub fn creation_stub(
        dependencies: solx_utils::Dependencies,
        payable: bool,
        source_file: &str,
        contract: (usize, usize),
        constructor: (usize, usize),
        emit_debug_info: bool,
    ) -> Self {
        let deploy_identifier = dependencies.identifier.as_str();
        let runtime_identifier = dependencies
            .runtime
            .as_deref()
            .expect("deploy code returns a runtime object");
        let text = if payable {
            include_str!("creation_stub_payable.ll")
        } else {
            include_str!("creation_stub.ll")
        }
        .replace("DEPLOY_OBJECT", deploy_identifier)
        .replace("RUNTIME_OBJECT", runtime_identifier)
        .replace("SOURCE_FILE", source_file)
        .replace("CONTRACT_LINE", contract.0.to_string().as_str())
        .replace("CONTRACT_COLUMN", contract.1.to_string().as_str())
        .replace("CONSTRUCTOR_LINE", constructor.0.to_string().as_str())
        .replace("CONSTRUCTOR_COLUMN", constructor.1.to_string().as_str());

        let llvm = inkwell::context::Context::create();
        let buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range_copy(
            text.as_bytes(),
            deploy_identifier,
        );
        let module = llvm
            .create_module_from_ir(buffer)
            .expect("the creation stub is a constant LLVM module");
        if emit_debug_info {
            module.add_basic_value_flag(
                "Dwarf Version",
                inkwell::module::FlagBehavior::Warning,
                llvm.i32_type().const_int(5, false),
            );
        } else {
            module.strip_debug_info();
        }

        Self {
            sol_source: None,
            source: None,
            bitcode: module.write_bitcode_to_memory().as_slice().to_vec(),
            dependencies,
        }
    }
}
