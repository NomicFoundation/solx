//!
//! The `solc --standard-json` output contract EVM bytecode link reference.
//!

///
/// The `solc --standard-json` output contract EVM bytecode link reference.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkReference {
    /// Start offset in the bytecode.
    pub start: u64,
    /// Length of the link reference.
    pub length: usize,
}

impl LinkReference {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(start: u64) -> Self {
        Self {
            start,
            length: solx_utils::BYTE_LENGTH_ETH_ADDRESS,
        }
    }
}
