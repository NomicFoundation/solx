//!
//! Whether a struct at the current position may stay opaque.
//!

/// Whether a struct at the current position may stay opaque.
pub enum Position {
    /// Embedded in its holder.
    ByValue,
    /// Behind an array, a mapping or a function reference.
    Breaking,
}
