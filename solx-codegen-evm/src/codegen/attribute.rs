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

impl std::fmt::Display for Attribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attribute::EVMEntryFunction => write!(f, "evm-entry-function"),
            Attribute::TargetFeatures => write!(f, "target-features"),
        }
    }
}
