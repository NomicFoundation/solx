//!
//! Stack-too-deep compilation error.
//!

///
/// Stack-too-deep compilation error.
///
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
#[error(
    "Stack-too-deep error. Required spill area: {spill_area_size:?} bytes; size fallback: {is_size_fallback}"
)]
pub struct StackTooDeep {
    /// Spill area size in bytes.
    pub spill_area_size: u64,
    /// Whether the size fallback was activated during the compilation.
    pub is_size_fallback: bool,
}
