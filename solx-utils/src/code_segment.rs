//!
//! Contract code segment.
//!

///
/// Contract code segment.
///
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum CodeSegment {
    /// The deploy code segment.
    Deploy,
    /// The runtime code segment.
    Runtime,
}

impl CodeSegment {
    /// The deploy bytecode size limit.
    pub const DEPLOY_CODE_SIZE_LIMIT: usize = 49152;

    /// The runtime bytecode size limit.
    pub const RUNTIME_CODE_SIZE_LIMIT: usize = 24576;

    ///
    /// The EVM bytecode size limit of the segment.
    ///
    pub const fn size_limit(self) -> usize {
        match self {
            Self::Deploy => Self::DEPLOY_CODE_SIZE_LIMIT,
            Self::Runtime => Self::RUNTIME_CODE_SIZE_LIMIT,
        }
    }
}

impl std::fmt::Display for CodeSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deploy => write!(f, "deploy"),
            Self::Runtime => write!(f, "runtime"),
        }
    }
}
