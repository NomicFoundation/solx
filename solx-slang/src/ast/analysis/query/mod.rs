//!
//! Pure-Slang queries (no MLIR): semantic facts solx computes for emission,
//! implemented as extension traits because the orphan rule requires a trait to
//! attach a method to a foreign Slang node.
//!

pub mod base_constructor_chain;
pub mod match_linearised_base;
pub mod member_access_operand;
pub mod method_identifiers;
pub mod modifier_resolution;
pub mod node_ids;
pub mod positional_arguments;
pub mod storage_layout;
