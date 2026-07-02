//!
//! solx codegen-analysis layer over Slang semantics.
//!
//! Semantic facts solx derives from the Slang AST for emission, exposed as stateless
//! [`query`] extension traits: orphan-rule shims that attach a method to a foreign Slang node.
//!

pub mod query;
pub mod walk;
