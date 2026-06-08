//!
//! Fallback-clause shape carried by a `sol.try`.
//!

/// The `catch` (fallback) clause a `sol.try` carries, selecting how its fallback
/// region is shaped. The op's lowering owns the selector dispatch, payload
/// decode, and raw re-revert, so the frontend only declares the clause's shape
/// here.
#[derive(Clone, Copy)]
pub enum TryFallbackKind {
    /// No `catch {}` / `catch (bytes)` clause: the fallback region is empty and
    /// the lowering re-reverts the raw revert data when no typed clause matches.
    None,
    /// Parameter-less `catch { ... }`: the region runs the body with no bound
    /// value.
    Parameterless,
    /// Low-level `catch (bytes memory data) { ... }`: the region binds the whole
    /// returndata as a memory `bytes` block argument.
    Bytes,
}
