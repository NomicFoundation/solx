//!
//! Groups test instances by the code generator version.
//!

pub mod versioned;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use self::versioned::VersionedGroup;

///
/// The language version associated with a test.
///
pub type Version = String;

///
/// Groups test instances by the code generator version.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CodegenGroup {
    #[serde(flatten)]
    /// Inner groups that differ by the associated language version.
    pub versioned_groups: BTreeMap<Option<Version>, VersionedGroup>,
}
