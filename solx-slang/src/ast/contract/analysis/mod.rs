//!
//! Pre-emission analysis pass over a contract.
//!
//! These hand-rolled AST walks compute what emission needs but Slang does not yet
//! expose: the free / library functions a contract reaches (so they are emitted
//! into its module under node-id-qualified symbols), and the C3-correct
//! `super` / virtual-dispatch redirects. They run once per contract and freeze
//! their results into the [`Context`](solx_mlir::Context) that emission reads.
//!
//! They are grouped here to be reviewed and, eventually, replaced together: a
//! slang `dev-solx` call-graph query (free / library reachability) and a
//! C3-correct super-resolution / override-chain query would retire these walks.
//! Until that fork API exists they stay as the analysis layer.

pub mod body_origin;
pub mod free_function;
pub mod library;
pub mod reachability;
pub mod super_call;
