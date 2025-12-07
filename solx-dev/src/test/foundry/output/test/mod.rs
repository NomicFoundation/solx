//!
//! Foundry test output report.
//!

pub mod file;

use std::collections::BTreeMap;

use self::file::File;

///
/// Foundry test output report.
///
#[derive(Debug, serde::Deserialize)]
pub struct Test(pub BTreeMap<String, File>);
