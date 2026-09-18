//!
//! The `solc --standard-json` input settings metadata.
//!

///
/// The `solc --standard-json` input settings metadata.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Whether to use literal content.
    #[serde(default)]
    pub use_literal_content: bool,

    /// Whether to append CBOR metadata.
    #[serde(rename = "appendCBOR", default = "Metadata::default_append_cbor")]
    pub append_cbor: bool,

    /// The metadata hash type.
    // Not sent to solc, which rejects `appendCBOR: false` with any `bytecodeHash` but `none`.
    #[serde(default = "Metadata::default_bytecode_hash", skip_serializing)]
    pub bytecode_hash: solx_utils::MetadataHashType,
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new(false, true, Self::default_bytecode_hash())
    }
}

impl Metadata {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        use_literal_content: bool,
        append_cbor: bool,
        hash_type: solx_utils::MetadataHashType,
    ) -> Self {
        Self {
            bytecode_hash: hash_type,
            append_cbor,
            use_literal_content,
        }
    }

    ///
    /// The default metadata hash type.
    ///
    fn default_bytecode_hash() -> solx_utils::MetadataHashType {
        solx_utils::MetadataHashType::IPFS
    }

    ///
    /// The default append CBOR flag.
    ///
    fn default_append_cbor() -> bool {
        true
    }
}
