//!
//! `solx` tester summary element outcome.
//!

pub mod passed_variant;

use crate::test::case::input::output::Output;

use self::passed_variant::PassedVariant;

///
/// `solx` tester summary element outcome.
///
#[derive(Debug)]
pub enum Outcome {
    /// The `passed` outcome.
    Passed {
        /// The outcome variant.
        variant: PassedVariant,
        /// The test group name.
        group: Option<String>,
    },
    /// The `failed` outcome. The output result is incorrect.
    Failed {
        /// The expected result.
        expected: Output,
        /// The actual result.
        found: Output,
        /// The calldata.
        calldata: String,
    },
    /// The `invalid` outcome. The test is incorrect.
    Invalid {
        /// The building error description.
        error: String,
    },
    /// The `ignored` outcome. The test is ignored.
    Ignored,
}

impl Outcome {
    ///
    /// A shortcut constructor.
    ///
    pub fn passed(group: Option<String>, variant: PassedVariant) -> Self {
        Self::Passed { group, variant }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn failed(expected: Output, found: Output, calldata: Vec<u8>) -> Self {
        Self::Failed {
            expected,
            found,
            calldata: hex::encode(calldata.as_slice()),
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn invalid<S>(error: S) -> Self
    where
        S: ToString,
    {
        Self::Invalid {
            error: error.to_string(),
        }
    }

    ///
    /// A shortcut constructor.
    ///
    pub fn ignored() -> Self {
        Self::Ignored
    }
}
