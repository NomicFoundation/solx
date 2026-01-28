//!
//! Contract compound name.
//!

///
/// Contract compound name.
///
/// Every language we support has a different way of defining the contract name.
/// This structure simplifies passing the contract name through the compilation pipeline.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractName {
    /// The absolute file path.
    pub path: String,
    /// The contract name.
    /// Is set for Solidity contracts only. Otherwise it would be equal to the file name.
    pub name: Option<String>,
    /// The full contract identifier.
    /// For Solidity, The format is `<absolute file path>:<contract name>`.
    /// For other languages, `<absolute file path>`.
    pub full_path: String,
}

impl ContractName {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(path: String, name: Option<String>) -> Self {
        let full_path = match name {
            Some(ref name) => format!("{path}:{name}"),
            None => path.clone(),
        };

        Self {
            path,
            name,
            full_path,
        }
    }
}
