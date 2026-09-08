//!
//! The Matter Labs compiler test metadata case input.
//!

pub mod calldata;
pub mod expected;
pub mod storage;

use std::collections::HashMap;

use crate::directories::matter_labs::test::default_caller_address;
use crate::directories::matter_labs::test::simple_tests_instance;

use self::calldata::Calldata;
use self::expected::Expected;
use self::storage::Storage;

///
/// The Matter Labs compiler test metadata case input.
///
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Input {
    /// The comment to an entry.
    pub comment: Option<String>,
    /// The contract instance.
    #[serde(default = "simple_tests_instance")]
    pub instance: String,
    /// The caller address.
    #[serde(default = "default_caller_address")]
    pub caller: String,
    /// The contract method name.
    /// `#deployer` for the deployer call
    /// `#fallback` for the fallback
    /// `#storage_empty` for asserting that the entire contract storage holds no non-zero word
    pub method: String,
    /// The passed calldata.
    pub calldata: Calldata,
    /// The passed value.
    pub value: Option<String>,
    /// The initial contracts storage.
    #[serde(default)]
    pub storage: HashMap<String, Storage>,

    /// The expected return data.
    pub expected: Option<Expected>,
}

impl Input {
    ///
    /// Creates a deployer call with empty constructor calldata.
    ///
    pub fn empty_deployer_call(instance: String) -> Self {
        Self {
            comment: None,
            instance: instance.clone(),
            caller: default_caller_address(),
            calldata: Calldata::default(),
            method: "#deployer".to_string(),
            value: None,
            storage: HashMap::new(),

            expected: Some(Expected::successful_deployer_expected(instance.clone())),
        }
    }
}
