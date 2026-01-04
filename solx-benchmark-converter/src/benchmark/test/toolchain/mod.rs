//!
//! Groups test instances by the toolchain.
//!

pub mod codegen;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use self::codegen::CodegenGroup;

///
/// The codegen identifier associated with a test.
///
pub type Codegen = String;

///
/// Groups test instances by the toolchain.
///
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ToolchainGroup {
    #[serde(flatten)]
    /// Inner groups that differ by the associated codegen.
    pub codegen_groups: BTreeMap<Option<Codegen>, CodegenGroup>,
}
