//!
//! Process for compiling a single compilation unit.
//!
//! The EVM output data.
//!

use crate::build::contract::object::Object as EVMContractObject;

///
/// The EVM output data.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Output {
    /// The contract build object.
    pub object: EVMContractObject,
}

impl Output {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(object: EVMContractObject) -> Self {
        Self { object }
    }
}
