//!
//! The `solc --standard-json` output contract EVM bytecode CBOR trailer.
//!

///
/// One CBOR metadata trailer in the bytecode.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CborAuxdata {
    /// The byte offset of the trailer from the start of the bytecode.
    pub offset: u64,
    /// The trailer bytes with the two trailing length bytes, as hexadecimal.
    pub value: String,
}

impl CborAuxdata {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(offset: u64, bytes: &[u8]) -> Self {
        Self {
            offset,
            value: format!("0x{}", hex::encode(bytes)),
        }
    }
}
