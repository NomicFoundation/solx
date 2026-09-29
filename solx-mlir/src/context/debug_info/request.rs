//!
//! Which of an object's code segments debug info is requested for.
//!

/// Which of an object's code segments debug info is requested for.
///
/// Both segments are one module until the pass pipeline splits them, so locations are emitted
/// when either asks; each segment's text carries them only when that segment asked.
#[derive(Clone, Copy)]
pub struct DebugInfoRequest {
    /// Debug info is requested for the deploy code.
    pub deploy: bool,
    /// Debug info is requested for the runtime code.
    pub runtime: bool,
}

impl DebugInfoRequest {
    /// Whether debug info is requested for either segment.
    pub fn any(self) -> bool {
        self.deploy || self.runtime
    }
}
