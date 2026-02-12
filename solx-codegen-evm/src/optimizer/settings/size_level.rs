//!
//! The LLVM optimizer settings size level.
//!

///
/// The LLVM optimizer settings size level.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SizeLevel {
    /// No size optimizations.
    Zero,
    /// The default size optimizations.
    S,
    /// The aggresize size optimizations.
    Z,
}
