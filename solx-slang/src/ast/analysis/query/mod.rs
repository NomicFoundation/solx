//!
//! Pure-Slang queries (no MLIR): semantic facts solx computes for emission,
//! implemented as extension traits because the orphan rule requires a trait to
//! attach a method to a foreign Slang node.
//!

pub mod method_identifiers;
pub mod storage_layout;

pub use self::method_identifiers::MethodIdentifiers;
pub use self::storage_layout::StorageLayout;
