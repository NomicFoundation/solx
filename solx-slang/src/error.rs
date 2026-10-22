//!
//! An output of a definition Slang accepts without diagnosing but cannot compute.
//!

/// An output of a definition Slang accepts without diagnosing but cannot compute, by the
/// definition's name.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The JSON ABI, such as of an interface with a contract base.
    #[error("Slang cannot compute the ABI of `{0}`.")]
    Abi(String),
    /// The storage layout, or on the codegen path the storage slots of persistent and transient
    /// state variables together.
    #[error("Slang cannot compute the storage layout of `{0}`.")]
    StorageLayout(String),
    /// The transient storage layout.
    #[error("Slang cannot compute the transient storage layout of `{0}`.")]
    TransientStorageLayout(String),
}
