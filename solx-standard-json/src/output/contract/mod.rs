//!
//! The `solc --standard-json` output contract.
//!

pub mod evm;

use self::evm::EVM;

///
/// The `solc --standard-json` output contract.
///
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    /// The contract ABI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abi: Option<serde_json::Value>,
    /// The contract storage layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_layout: Option<serde_json::Value>,
    /// The contract storage layout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transient_storage_layout: Option<serde_json::Value>,
    /// The contract metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    /// The contract developer documentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub devdoc: Option<serde_json::Value>,
    /// The contract user documentation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub userdoc: Option<serde_json::Value>,
    /// The contract Yul IR code.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ir: Option<String>,
    /// The EVM data of the contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evm: Option<EVM>,
}

impl Contract {
    ///
    /// Checks if all fields are unset or empty.
    ///
    pub fn is_empty(&self) -> bool {
        self.abi.is_none()
            && self.storage_layout.is_none()
            && self.transient_storage_layout.is_none()
            && self.metadata.is_none()
            && self.devdoc.is_none()
            && self.userdoc.is_none()
            && self.ir.is_none()
            && match self.evm.as_ref() {
                Some(evm) => evm.is_empty(),
                None => true,
            }
    }
}
