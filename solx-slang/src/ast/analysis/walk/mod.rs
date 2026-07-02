//!
//! Pre-emission analysis walks over a contract.
//!
//! These hand-rolled AST walks compute what emission needs from Slang semantics:
//! the free / library functions a contract reaches (so they are emitted into its
//! module under node-id-qualified symbols), and the C3-correct `super` /
//! virtual-dispatch redirects. They run once per contract and freeze their
//! results into the [`Context`](solx_mlir::Context) that emission reads.

pub mod body_origin;
pub mod free_function;
pub mod library;
pub mod reachability;
pub mod super_call;
