//!
//! One row's movement between the main and PR toolchains.
//!

///
/// One row's movement between the main and PR toolchains.
///
pub struct Movement {
    pub label: String,
    pub mode: String,
    pub main: u64,
    pub pr: u64,
}
