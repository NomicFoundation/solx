//!
//! Sol dialect attribute enums for MLIR code generation.
//!

pub mod contract_kind;
pub mod icmp_predicate;
pub mod state_mutability;

pub use self::contract_kind::ContractKind;
pub use self::icmp_predicate::ICmpPredicate;
pub use self::state_mutability::StateMutability;
