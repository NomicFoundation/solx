//!
//! Whether a library function call is internal (inlined) or external
//! (delegatecalled).
//!

/// The visibility of a called library function, which determines how the call
/// lowers. Replaces an `external: bool` flag.
pub enum LibraryVisibility {
    /// An `internal` library function — inlined at the call site.
    Internal,
    /// An `external` / `public` library function (it has an ABI selector) —
    /// delegatecalled to the deployed library.
    External,
}
