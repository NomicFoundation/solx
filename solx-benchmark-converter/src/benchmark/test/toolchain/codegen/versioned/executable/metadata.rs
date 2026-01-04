//!
//! Information associated with an executable in a benchmark.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// Information associated with an executable in a benchmark.
///
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {}
