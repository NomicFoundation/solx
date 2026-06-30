//!
//! Synthesis of the external accessor for a `public` state variable: the body emission, its ABI
//! signature, and the struct-member layout the body and the signature share.
//!

pub mod emit;
pub mod keyed_signature;
pub mod member;
pub mod signature;

pub use self::signature::Signature;
