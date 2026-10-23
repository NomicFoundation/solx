//!
//! The contract metadata the frontend knows: the compiled contract, its ABI, the sources it imports
//! and the frontend settings. The compiler adds its own version and the code generation settings
//! before hashing it into the bytecode.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::utils::LanguageVersion;

use solx_utils::EVMVersion;

use crate::abi::AbiDefinition;

/// The metadata settings every contract of a compilation unit shares, and the sources its
/// contracts import.
pub struct Metadata<'input> {
    /// The unit whose import graph selects each contract's sources.
    unit: &'input CompilationUnit,
    /// The input sources, hashed or embedded into each contract's `sources`.
    input_sources: &'input BTreeMap<String, solx_standard_json::InputSource>,
    /// Whether `sources` embeds the content instead of its IPFS URL.
    use_literal_content: bool,
    /// The settings every contract shares.
    settings: serde_json::Map<String, serde_json::Value>,
}

impl<'input> Metadata<'input> {
    /// Collects the settings of `input_settings` that change what the frontend produces, with the
    /// EVM and language versions resolved.
    pub fn new(
        unit: &'input CompilationUnit,
        input_sources: &'input BTreeMap<String, solx_standard_json::InputSource>,
        input_settings: &solx_standard_json::InputSettings,
        evm_version: EVMVersion,
        language_version: LanguageVersion,
    ) -> Self {
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "bytecodeHash".to_owned(),
            serde_json::to_value(input_settings.metadata.bytecode_hash)
                .expect("a hash type serializes to a string"),
        );
        if !input_settings.metadata.append_cbor {
            metadata.insert("appendCBOR".to_owned(), false.into());
        }
        if input_settings.metadata.use_literal_content {
            metadata.insert("useLiteralContent".to_owned(), true.into());
        }

        let libraries = input_settings
            .libraries
            .as_inner()
            .iter()
            .flat_map(|(path, libraries)| {
                libraries
                    .iter()
                    .map(move |(name, address)| (format!("{path}:{name}"), address.as_str().into()))
            })
            .collect::<serde_json::Map<_, _>>();

        let mut remappings = input_settings
            .remappings
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        remappings.sort();

        let mut settings = serde_json::Map::new();
        settings.insert("evmVersion".to_owned(), evm_version.to_string().into());
        settings.insert(
            "solidityVersion".to_owned(),
            semver::Version::from(language_version).to_string().into(),
        );
        settings.insert("libraries".to_owned(), libraries.into());
        settings.insert("metadata".to_owned(), metadata.into());
        settings.insert("remappings".to_owned(), remappings.into());

        Self {
            unit,
            input_sources,
            use_literal_content: input_settings.metadata.use_literal_content,
            settings,
        }
    }

    /// The metadata of `definition`, whose ABI is `abi`. `sources` holds the definition's file
    /// and every file it imports, directly or not, so files compiled alongside leave it unchanged.
    pub fn contract(&self, definition: &AbiDefinition, abi: &serde_json::Value) -> String {
        let file_id = definition.file_id();
        let sources = self
            .unit
            .compute_file_dependencies(file_id)
            .expect("Slang computes the dependencies of every file of the unit")
            .iter()
            .map(|dependency| {
                let content = self
                    .input_sources
                    .get(dependency.as_str())
                    .and_then(solx_standard_json::InputSource::content)
                    .expect("every file of the unit is an input source with content");
                let keccak256 = solx_utils::Keccak256Hash::from_slice(content.as_bytes());
                let source = if self.use_literal_content {
                    serde_json::json!({ "keccak256": keccak256.as_str(), "content": content })
                } else {
                    let ipfs = solx_utils::IPFSHash::from_slice(content.as_bytes());
                    serde_json::json!({
                        "keccak256": keccak256.as_str(),
                        "urls": [format!("dweb:/ipfs/{}", ipfs.as_base58())],
                    })
                };
                (dependency.to_string(), source)
            })
            .collect::<serde_json::Map<_, _>>();

        let mut settings = self.settings.clone();
        settings.insert(
            "compilationTarget".to_owned(),
            serde_json::json!({ file_id.as_str(): definition.name().name() }),
        );

        serde_json::json!({
            "language": "Solidity",
            "output": { "abi": abi },
            "settings": settings,
            "sources": sources,
            "version": 1,
        })
        .to_string()
    }
}
