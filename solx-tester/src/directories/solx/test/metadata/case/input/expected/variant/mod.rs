//!
//! The solx compiler test metadata expected data variant.
//!

pub mod extended;

use serde::Deserialize;

use self::extended::Extended;

///
/// The solx compiler test metadata expected data variant.
///
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Variant {
    /// The return values only list.
    Simple(Vec<String>),
    /// The extended snapshot data testing.
    Extended(Extended),
}
