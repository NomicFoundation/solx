//!
//! The `solc --standard-json` output contract EVM bytecode immutable reference.
//!

///
/// The `solc --standard-json` output contract EVM bytecode immutable reference.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImmutableReference {
    /// Start offset in the bytecode.
    pub start: u64,
    /// Length of the immutable value.
    pub length: usize,
}

impl ImmutableReference {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(start: u64) -> Self {
        Self {
            start,
            length: solx_utils::BYTE_LENGTH_FIELD,
        }
    }
}
