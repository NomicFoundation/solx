//!
//! One row's movement between the main and PR toolchains.
//!

///
/// One row's movement between the main and PR toolchains.
///
pub(crate) struct Movement {
    pub(crate) label: String,
    pub(crate) mode: String,
    pub(crate) main: u64,
    pub(crate) pr: u64,
}
