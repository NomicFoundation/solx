//!
//! Identifier for the test input. Describes the input type and position but not the actual contents.
//!

use serde::Deserialize;
use serde::Serialize;

///
/// Identifier for the test input. Describes the input type and position but not the actual contents.
///
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Input {
    /// The contract deploy, regardless of target.
    Deployer {
        /// Contract identifier, usually file name and contract name separated by a colon.
        contract_identifier: String,
    },
    /// The fallback method.
    Fallback {
        /// Index in the array of inputs.
        input_index: usize,
    },
    /// The contract call.
    Runtime {
        /// Index in the array of inputs.
        input_index: usize,
        /// Input name, provided in the test description.
        name: String,
    },
    /// The storage empty check.
    StorageEmpty {
        /// Index in the array of inputs.
        input_index: usize,
    },
    /// Check account balance.
    Balance {
        /// Index in the array of inputs.
        input_index: usize,
    },
}

impl Input {
    /// Returns `true` if the input is [`Deployer`].
    ///
    /// [`Deployer`]: Input::Deployer
    #[must_use]
    pub fn is_deploy(&self) -> bool {
        matches!(self, Self::Deployer { .. })
    }

    /// Returns `true` if the input is [`Fallback`].
    ///
    /// [`Fallback`]: Input::Fallback
    #[must_use]
    pub fn is_fallback(&self) -> bool {
        matches!(self, Self::Fallback { .. })
    }

    ///
    /// Returns the runtime function name if it is applicable.
    ///
    pub fn runtime_name(&self) -> Option<&str> {
        match self {
            Self::Runtime { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Input::Deployer {
                contract_identifier,
            } => f.write_fmt(format_args!("#deployer:{contract_identifier}")),
            Input::Runtime { input_index, name } => {
                f.write_fmt(format_args!("{name}:{input_index}"))
            }
            Input::StorageEmpty { input_index } => {
                f.write_fmt(format_args!("#storage_empty_check:{input_index}"))
            }
            Input::Balance { input_index } => {
                f.write_fmt(format_args!("#balance_check:{input_index}"))
            }
            Input::Fallback { input_index } => f.write_fmt(format_args!("#fallback:{input_index}")),
        }
    }
}
