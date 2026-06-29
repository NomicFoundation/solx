//!
//! Fallback-clause shape carried by a `sol.try`.
//!

/// The `catch` (fallback) clause a `sol.try` carries, selecting how its fallback region is shaped.
#[derive(Clone, Copy)]
pub enum TryFallbackKind {
    /// No `catch {}` / `catch (bytes)` clause: the region is empty and the conversion re-reverts raw revert data.
    None,
    /// Empty `catch { ... }`: the region runs the body with no bound value.
    Empty,
    /// Low-level `catch (bytes memory data) { ... }`: the region binds the returndata as a memory `bytes` argument.
    Bytes,
}
