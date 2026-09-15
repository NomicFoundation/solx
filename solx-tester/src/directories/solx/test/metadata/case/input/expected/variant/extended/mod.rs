//!
//! The solx compiler test metadata extended expected data.
//!

pub mod event;

use serde::Deserialize;

use self::event::Event;

///
/// The solx compiler test metadata extended expected data.
///
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Extended {
    /// The return data values.
    pub return_data: Vec<String>,
    /// The emitted events.
    #[serde(default)]
    pub events: Vec<Event>,
    /// Whether an exception is expected,
    #[serde(default)]
    pub exception: bool,
    /// The compiler version filter.
    pub compiler_version: Option<semver::VersionReq>,
}
