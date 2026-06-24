//!
//! solx codegen-analysis layer over Slang semantics.
//!
//! Semantic facts solx derives from the Slang AST for emission, in two shapes:
//! [`query`] — stateless extension-trait queries (orphan-rule shims over foreign
//! Slang nodes); [`walk`] — stateful per-contract analysis walks whose results
//! freeze into the emission [`Context`](solx_mlir::Context).
//!

pub mod query;
pub mod walk;
