//!
//! Pure-Slang queries (no MLIR): semantic facts solx computes for emission,
//! implemented as extension traits because the orphan rule requires a trait to
//! attach a method to a foreign Slang node.
//!

pub mod match_linearised_base;
pub mod member_access_operand;
pub mod method_identifiers;
pub mod modifier_resolution;
pub mod immutable_storage_layout;
pub mod positional_arguments;
pub mod storage_layout;

pub use self::match_linearised_base::MatchLinearisedBase;
pub use self::member_access_operand::MemberAccessOperand;
pub use self::method_identifiers::MethodIdentifiers;
pub use self::modifier_resolution::ModifierResolution;
pub use self::immutable_storage_layout::ImmutableStorageLayout;
pub use self::positional_arguments::PositionalArguments;
pub use self::storage_layout::StorageLayout;
