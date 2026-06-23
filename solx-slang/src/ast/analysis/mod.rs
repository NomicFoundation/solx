//!
//! solx codegen-analysis layer over Slang semantics.
//!
//! Semantic facts solx derives from the Slang AST for emission: [`query`] —
//! stateless extension-trait queries (orphan-rule shims over foreign Slang nodes).
//!

pub mod query;
