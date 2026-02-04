//!
//! Bytecode object.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

///
/// Bytecode object.
///
/// Can be either deploy and runtime code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Object {
    /// Object identifier.
    pub identifier: String,
    /// Contract full name.
    pub contract_name: solx_utils::ContractName,
    /// Text assembly.
    pub assembly: Option<String>,
    /// Bytecode.
    pub bytecode: Option<Vec<u8>>,
    /// Hexadecimal bytecode.
    pub bytecode_hex: Option<String>,
    /// Debug info.
    pub debug_info: Option<Vec<u8>>,
    /// EVM legacy assembly IR (solx internal representation).
    pub evmla: Option<String>,
    /// Ethereal IR (solx internal representation).
    pub ethir: Option<String>,
    /// Unoptimized LLVM IR (solx internal representation).
    pub llvm_ir_unoptimized: Option<String>,
    /// Optimized LLVM IR (solx internal representation).
    pub llvm_ir: Option<String>,
    /// Whether IR codegen is used.
    pub via_ir: bool,
    /// Code segment.
    pub code_segment: solx_utils::CodeSegment,
    /// The metadata bytes. Only appended to runtime code.
    pub metadata_bytes: Option<Vec<u8>>,
    /// Immutables of the runtime code.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// Dependencies.
    pub dependencies: solx_codegen_evm::Dependencies,
    /// Linker symbols that were not provided at compile time.
    /// Such symbols must be resolved by tooling before deployment.
    pub unlinked_symbols: BTreeMap<String, Vec<u64>>,
    /// Whether the object is already assembled.
    pub is_assembled: bool,
    /// Whether the size fallback was activated during the compilation.
    pub is_size_fallback: bool,
    /// Compilation warnings.
    pub warnings: Vec<solx_codegen_evm::Warning>,
    /// Compilation pipeline benchmarks.
    pub benchmarks: Vec<(String, u64)>,
}

impl Object {
    /// Length of the library placeholder.
    pub const LIBRARY_PLACEHOLDER_LENGTH: usize = 17;

    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        identifier: String,
        contract_name: solx_utils::ContractName,
        assembly: Option<String>,
        bytecode: Option<Vec<u8>>,
        debug_info: Option<Vec<u8>>,
        evmla: Option<String>,
        ethir: Option<String>,
        llvm_ir_unoptimized: Option<String>,
        llvm_ir: Option<String>,
        via_ir: bool,
        code_segment: solx_utils::CodeSegment,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        metadata_bytes: Option<Vec<u8>>,
        dependencies: solx_codegen_evm::Dependencies,
        is_size_fallback: bool,
        warnings: Vec<solx_codegen_evm::Warning>,
        benchmarks: Vec<(String, u64)>,
    ) -> Self {
        let bytecode_hex = bytecode.as_ref().map(hex::encode);
        Self {
            identifier,
            contract_name,
            assembly,
            bytecode,
            bytecode_hex,
            debug_info,
            evmla,
            ethir,
            llvm_ir_unoptimized,
            llvm_ir,
            via_ir,
            code_segment,
            immutables,
            metadata_bytes,
            dependencies,
            unlinked_symbols: BTreeMap::new(),
            is_assembled: false,
            is_size_fallback,
            warnings,
            benchmarks,
        }
    }

    ///
    /// Appends metadata to the object.
    ///
    /// # Panics
    /// If bytecode is `None`.
    ///
    pub fn to_memory_buffer(&self) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
        let bytecode = self.bytecode.as_deref().expect("Bytecode is not set");

        let mut memory_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
            bytecode,
            self.identifier.as_str(),
            false,
        );

        if let (solx_utils::CodeSegment::Runtime, Some(metadata_bytes)) =
            (self.code_segment, &self.metadata_bytes)
        {
            memory_buffer =
                solx_codegen_evm::append_metadata(memory_buffer, metadata_bytes.as_slice())?;
        }

        Ok(memory_buffer)
    }

    ///
    /// Assembles the object.
    ///
    /// # Panics
    /// If bytecode is `None`.
    ///
    pub fn assemble(
        &self,
        all_objects: &[&Self],
    ) -> anyhow::Result<inkwell::memory_buffer::MemoryBuffer> {
        let memory_buffer = self.to_memory_buffer()?;

        let mut memory_buffers = Vec::with_capacity(1 + self.dependencies.inner.len());
        memory_buffers.push((self.identifier.to_owned(), memory_buffer));

        memory_buffers.extend(self.dependencies.inner.iter().map(|dependency| {
            let original_dependency_identifier = dependency.to_owned();
            let dependency = all_objects
                .iter()
                .find(|object| object.identifier.as_str() == dependency.as_str())
                .expect("Dependency not found");
            let dependency_bytecode = dependency.bytecode.as_deref().expect("Bytecode is not set");
            let memory_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                dependency_bytecode,
                dependency.identifier.as_str(),
                false,
            );
            (original_dependency_identifier, memory_buffer)
        }));

        let bytecode_buffers = memory_buffers
            .iter()
            .map(|(_identifier, memory_buffer)| memory_buffer)
            .collect::<Vec<&inkwell::memory_buffer::MemoryBuffer>>();
        let bytecode_ids = memory_buffers
            .iter()
            .map(|(identifier, _memory_buffer)| identifier.as_str())
            .collect::<Vec<&str>>();
        solx_codegen_evm::assemble(
            bytecode_buffers.as_slice(),
            bytecode_ids.as_slice(),
            self.code_segment,
        )
    }

    ///
    /// Links the object with its linker symbols.
    ///
    /// # Panics
    /// If bytecode is `None`.
    ///
    pub fn link(
        &mut self,
        linker_symbols: &BTreeMap<String, [u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS]>,
    ) -> anyhow::Result<()> {
        let memory_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
            self.bytecode.as_deref().expect("Bytecode is not set"),
            self.identifier.as_str(),
            false,
        );

        let linked_object = solx_codegen_evm::link(memory_buffer, linker_symbols)?;
        let linked_object_with_placeholders = if !self.unlinked_symbols.is_empty() {
            let linker_placeholders = self
                .unlinked_symbols
                .keys()
                .map(|symbol| {
                    (
                        symbol.to_owned(),
                        [0u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS],
                    )
                })
                .collect::<BTreeMap<String, [u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS]>>();
            solx_codegen_evm::link(linked_object, &linker_placeholders)?
        } else {
            linked_object
        };

        let mut bytecode_hex = hex::encode(linked_object_with_placeholders.as_slice());
        for (symbol, offsets) in self.unlinked_symbols.iter() {
            let hash = solx_utils::Keccak256Hash::from_slice(symbol.as_bytes()).to_vec();
            let placeholder = format!(
                "__${}$__",
                hex::encode(&hash[0..Self::LIBRARY_PLACEHOLDER_LENGTH])
            );
            for offset in offsets.iter() {
                let offset = *offset as usize;
                unsafe {
                    bytecode_hex.as_bytes_mut()
                        [(offset * 2)..(offset + solx_utils::BYTE_LENGTH_ETH_ADDRESS) * 2]
                        .copy_from_slice(placeholder.as_bytes());
                }
            }
        }
        self.bytecode = Some(linked_object_with_placeholders.as_slice().to_owned());
        self.bytecode_hex = Some(bytecode_hex);

        Ok(())
    }

    ///
    /// Extracts warnings in standard JSON format.
    ///
    pub fn take_warnings_standard_json(&mut self) -> Vec<solx_standard_json::OutputError> {
        self.warnings
            .drain(..)
            .map(|warning| {
                solx_standard_json::OutputError::new_warning_with_data(
                    Some(self.contract_name.path.as_str()),
                    warning.code(),
                    warning.to_string(),
                    None,
                    None,
                )
            })
            .collect::<Vec<solx_standard_json::OutputError>>()
    }
}
