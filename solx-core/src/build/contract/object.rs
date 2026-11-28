//!
//! Bytecode object.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use solx_standard_json::OutputError;

///
/// One CBOR metadata trailer in the bytecode.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CborAuxdataEntry {
    /// The full name of the contract owning the trailer.
    pub contract_name: String,
    /// The byte offset of the trailer from the start of the bytecode.
    pub offset: u64,
    /// The trailer length including the two trailing length bytes.
    pub length: u64,
}

///
/// An assembled object ready for embedding: its relocatable ELF and the CBOR
/// trailers of its text.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssembledObject {
    /// The assembled ELF.
    #[serde(with = "serde_bytes")]
    pub bytecode: Vec<u8>,
    /// The CBOR metadata trailers, with offsets into the ELF's text.
    pub cbor_auxdata: Vec<CborAuxdataEntry>,
}

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
    #[serde(with = "serde_bytes")]
    pub bytecode: Option<Vec<u8>>,
    /// ELF-wrapped DWARF debug info.
    #[serde(with = "serde_bytes")]
    pub debug_info: Option<Vec<u8>>,
    /// Unoptimized LLVM IR (solx internal representation).
    pub llvm_ir_unoptimized: Option<String>,
    /// Optimized LLVM IR (solx internal representation).
    pub llvm_ir: Option<String>,
    /// The metadata bytes. Only appended to runtime code.
    #[serde(with = "serde_bytes")]
    pub metadata_bytes: Option<Vec<u8>>,
    /// Immutables of the runtime code.
    pub immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
    /// Dependencies.
    pub dependencies: solx_utils::Dependencies,
    /// Linker symbols that were not provided at compile time.
    /// Such symbols must be resolved by tooling before deployment.
    pub unlinked_symbols: BTreeMap<String, Vec<u64>>,
    /// The assembled ELF retained after worker linking for objects that embed this one.
    /// `Build::link` takes it and drops it once every object is linked.
    #[serde(with = "serde_bytes")]
    pub assembled_bytecode: Option<Vec<u8>>,
    /// Every CBOR metadata trailer in the bytecode.
    pub cbor_auxdata: Vec<CborAuxdataEntry>,
    /// Whether the size fallback was activated during the compilation.
    pub is_size_fallback: bool,
    /// Compilation warnings.
    pub warnings: Vec<solx_utils::Warning>,
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
        llvm_ir_unoptimized: Option<String>,
        llvm_ir: Option<String>,
        immutables: Option<BTreeMap<String, BTreeSet<u64>>>,
        metadata_bytes: Option<Vec<u8>>,
        dependencies: solx_utils::Dependencies,
        is_size_fallback: bool,
        warnings: Vec<solx_utils::Warning>,
    ) -> Self {
        Self {
            identifier,
            contract_name,
            assembly,
            bytecode,
            debug_info,
            llvm_ir_unoptimized,
            llvm_ir,
            immutables,
            metadata_bytes,
            dependencies,
            unlinked_symbols: BTreeMap::new(),
            assembled_bytecode: None,
            cbor_auxdata: Vec::new(),
            is_size_fallback,
            warnings,
            benchmarks: Vec::new(),
        }
    }

    ///
    /// Returns the available embedded objects, or `None` when the parent must finish assembly.
    ///
    pub fn embedded_objects(
        &self,
        runtime_assembled_object: Option<AssembledObject>,
    ) -> Option<BTreeMap<String, AssembledObject>> {
        if !self.dependencies.objects.is_empty() {
            return None;
        }
        match (self.dependencies.runtime.as_ref(), runtime_assembled_object) {
            (None, None) => Some(BTreeMap::new()),
            (None, Some(_)) => {
                unreachable!("Runtime objects are supplied only to deploy objects")
            }
            (Some(identifier), Some(runtime_assembled_object)) => Some(BTreeMap::from([(
                identifier.to_owned(),
                runtime_assembled_object,
            )])),
            (Some(_), None) => None,
        }
    }

    ///
    /// Assembles the object with the objects it embeds and links it with `linker_symbols`.
    ///
    /// # Panics
    /// If bytecode is `None` or an embedded object is missing.
    ///
    pub fn assemble_and_link(
        &mut self,
        assembled_by_identifier: &BTreeMap<String, AssembledObject>,
        linker_symbols: &BTreeMap<String, [u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS]>,
    ) {
        let mut memory_buffers = Vec::with_capacity(
            1 + usize::from(self.dependencies.runtime.is_some()) + self.dependencies.objects.len(),
        );
        memory_buffers.push((
            self.identifier.to_owned(),
            inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                self.bytecode.as_deref().expect("Bytecode is not set"),
                self.identifier.as_str(),
                false,
            ),
        ));

        let mut embedded_objects = Vec::with_capacity(self.dependencies.into_iter().count());
        for dependency in self.dependencies.into_iter() {
            // TODO: unreachable once Slang rejects `new` on an abstract contract (NomicFoundation/slang-diagnostics-research#1124).
            let embedded = assembled_by_identifier
                .get(dependency)
                .expect("Every dependency is assembled before the objects embedding it");
            let memory_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                embedded.bytecode.as_slice(),
                dependency.as_str(),
                false,
            );
            memory_buffers.push((dependency.to_owned(), memory_buffer));
            embedded_objects.push(embedded);
        }

        let bytecode_buffers = memory_buffers
            .iter()
            .map(|(_identifier, memory_buffer)| memory_buffer)
            .collect::<Vec<&inkwell::memory_buffer::MemoryBuffer>>();
        let bytecode_ids = memory_buffers
            .iter()
            .map(|(identifier, _memory_buffer)| identifier.as_str())
            .collect::<Vec<&str>>();
        let result = inkwell::memory_buffer::MemoryBuffer::assemble_evm(
            bytecode_buffers.as_slice(),
            bytecode_ids.as_slice(),
            self.metadata_bytes.as_deref().unwrap_or_default(),
            linker_symbols,
            true,
        );
        self.assembled_bytecode = Some(
            result
                .elf_object
                .expect("Always requested")
                .as_slice()
                .to_owned(),
        );

        // The layout's first entry is this object's own text.
        let mut cbor_auxdata = Vec::new();
        for (embedded, layout) in embedded_objects
            .iter()
            .zip(result.input_layout.iter().skip(1))
        {
            let Some(object_offset) = layout else {
                continue;
            };
            for entry in embedded.cbor_auxdata.iter() {
                cbor_auxdata.push(CborAuxdataEntry {
                    contract_name: entry.contract_name.clone(),
                    offset: object_offset + entry.offset,
                    length: entry.length,
                });
            }
        }
        if let Some(metadata_bytes) = self.metadata_bytes.as_ref() {
            cbor_auxdata.push(CborAuxdataEntry {
                contract_name: self.contract_name.full_path.to_owned(),
                offset: result.metadata_offset,
                length: metadata_bytes.len() as u64,
            });
        }
        self.cbor_auxdata = cbor_auxdata;
        for (reference, offset) in result.unresolved_references.into_iter() {
            self.unlinked_symbols
                .entry(reference)
                .or_default()
                .push(offset);
        }
        self.bytecode = Some(result.bytecode.as_slice().to_owned());
    }

    ///
    /// Encodes the linked bytecode as hex, with the placeholder solc emits in
    /// place of every library address still unknown.
    ///
    /// # Panics
    /// If bytecode is `None`.
    ///
    pub fn to_bytecode_hex(&self) -> String {
        let mut bytecode_hex = hex::encode(
            self.bytecode
                .as_deref()
                .expect("Bytecode is emitted whenever its output is selected"),
        );
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
        bytecode_hex
    }

    ///
    /// Extracts warnings in standard JSON format.
    ///
    pub fn take_warnings_standard_json(&mut self) -> Vec<OutputError> {
        self.warnings
            .drain(..)
            .map(|warning| {
                OutputError::new_warning_with_data(
                    Some(self.contract_name.path.as_str()),
                    warning.code(),
                    warning.to_string(),
                    None,
                    None,
                )
            })
            .collect::<Vec<OutputError>>()
    }
}
