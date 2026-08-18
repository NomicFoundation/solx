//!
//! The EVM string attribute.
//!

///
/// The EVM string attribute.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum Attribute {
    /// The corresponding value.
    EVMEntryFunction,
    /// The corresponding value.
    TargetFeatures,
}

impl Attribute {
    ///
    /// Returns the LLVM attribute key.
    ///
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EVMEntryFunction => "evm-entry-function",
            Self::TargetFeatures => "target-features",
        }
    }
}
