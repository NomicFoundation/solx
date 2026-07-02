//!
//! Synthesis of the external accessor for a `public` state variable: the body emission, its ABI
//! signature, and the struct-member layout the body and the signature share.
//!

/// Emits the external accessor body for a `public` state variable.
pub mod emit;
/// The resolved signature of a keyed (`mapping`/array) getter.
pub mod keyed_signature;
/// A returnable member of a struct getter.
pub mod member;
/// The external ABI signature of a `public` state variable's synthesised getter.
pub mod signature;
