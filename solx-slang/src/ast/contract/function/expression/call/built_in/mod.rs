//!
//! Solidity built-in function and EVM intrinsic emission.
//!

pub mod abi;
pub mod array;

/// ABI encoding mode for `abi.encode` / `abi.encodePacked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    /// Standard ABI encoding with per-element padding (`abi.encode`,
    /// `abi.encodeWithSelector`, `abi.encodeWithSignature`).
    Standard,
    /// Packed ABI encoding with no per-element padding (`abi.encodePacked`).
    Packed,
}
