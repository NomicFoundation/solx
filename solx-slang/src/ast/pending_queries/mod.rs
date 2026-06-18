//!
//! PROVISIONAL holding for pure-Slang queries (no MLIR) lifted off the dissolved
//! god-emitters. Their real home — a Slang `dev-solx` node_extension, a solx
//! concept, or folded into a caller — is decided in a dedicated query-sorting
//! pass, not here. They live as traits only because the orphan rule requires a
//! trait to attach a method to a foreign Slang node; the trait is a parking spot,
//! not a design claim.
//!

pub mod match_linearised_base;
pub mod method_identifiers;
pub mod modifier_resolution;
pub mod positional_arguments;
pub mod storage_layout;

pub use self::match_linearised_base::MatchLinearisedBase;
pub use self::method_identifiers::MethodIdentifiers;
pub use self::modifier_resolution::ModifierResolution;
pub use self::positional_arguments::PositionalArguments;
pub use self::storage_layout::StorageLayout;
