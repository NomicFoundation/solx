//!
//! Bytecode object.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use solx_standard_json::OutputError;

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
        runtime_assembled_bytecode: Option<Vec<u8>>,
    ) -> Option<BTreeMap<String, Vec<u8>>> {
        if !self.dependencies.objects.is_empty() {
            return None;
        }
        match (
            self.dependencies.runtime.as_ref(),
            runtime_assembled_bytecode,
        ) {
            (None, None) => Some(BTreeMap::new()),
            (None, Some(_)) => {
                unreachable!("Runtime bytes are supplied only to deploy objects")
            }
            (Some(identifier), Some(runtime_assembled_bytecode)) => Some(BTreeMap::from([(
                identifier.to_owned(),
                runtime_assembled_bytecode,
            )])),
            (Some(_), None) => None,
        }
    }

    ///
    /// Assembles the object with the objects it embeds, links it with `linker_symbols`, and
    /// records the library references they leave unresolved.
    ///
    /// # Panics
    /// If bytecode is `None` or an embedded object is missing.
    ///
    pub fn assemble_and_link(
        &mut self,
        assembled_by_identifier: &BTreeMap<String, Vec<u8>>,
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

        for dependency in self.dependencies.into_iter() {
            // TODO: unreachable once Slang rejects `new` on an abstract contract (NomicFoundation/slang-diagnostics-research#1124).
            let bytecode = assembled_by_identifier
                .get(dependency)
                .expect("Every dependency is assembled before the objects embedding it");
            let memory_buffer = inkwell::memory_buffer::MemoryBuffer::create_from_memory_range(
                bytecode.as_slice(),
                dependency.as_str(),
                false,
            );
            memory_buffers.push((dependency.to_owned(), memory_buffer));
        }

        let bytecode_buffers = memory_buffers
            .iter()
            .map(|(_identifier, memory_buffer)| memory_buffer)
            .collect::<Vec<&inkwell::memory_buffer::MemoryBuffer>>();
        let bytecode_ids = memory_buffers
            .iter()
            .map(|(identifier, _memory_buffer)| identifier.as_str())
            .collect::<Vec<&str>>();
        let assembled = inkwell::memory_buffer::MemoryBuffer::assemble_evm(
            bytecode_buffers.as_slice(),
            bytecode_ids.as_slice(),
            self.metadata_bytes.as_deref().unwrap_or_default(),
        );
        self.assembled_bytecode = Some(assembled.as_slice().to_owned());

        self.unlinked_symbols = assembled
            .get_undefined_references_evm()
            .into_iter()
            .filter(|reference| !linker_symbols.contains_key(reference))
            .map(|reference| {
                let offsets = assembled.get_symbol_offsets_evm(reference.as_str());
                (reference, offsets)
            })
            .collect();
        let linked_object = if self.unlinked_symbols.is_empty() {
            assembled.link_evm(linker_symbols)
        } else {
            let mut symbols = linker_symbols.clone();
            for symbol in self.unlinked_symbols.keys() {
                symbols.insert(
                    symbol.to_owned(),
                    [0u8; solx_utils::BYTE_LENGTH_ETH_ADDRESS],
                );
            }
            assembled.link_evm(&symbols)
        };

        self.bytecode = Some(linked_object.as_slice().to_owned());
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
