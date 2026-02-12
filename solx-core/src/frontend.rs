//!
//! Solidity frontend interface trait.
//!

use std::path::PathBuf;

///
/// Solidity frontend interface trait.
///
pub trait Frontend {
    ///
    /// Returns the frontend compiler name.
    ///
    fn name(&self) -> &str;

    ///
    /// The Solidity `--standard-json` mirror.
    ///
    /// Metadata is always requested in order to calculate the metadata hash even if not requested in the `output_selection`.
    /// EVM assembly or Yul is always selected in order to compile the Solidity code.
    ///
    fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
        use_import_callback: bool,
        base_path: Option<&str>,
        include_paths: &[String],
        allow_paths: Option<String>,
    ) -> anyhow::Result<solx_standard_json::Output>;

    ///
    /// Validates the Yul project as paths and libraries.
    ///
    fn validate_yul_paths(
        &self,
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
    ) -> anyhow::Result<solx_standard_json::Output>;

    ///
    /// Validates the Yul project as standard JSON input.
    ///
    fn validate_yul_standard_json(
        &self,
        solc_input: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output>;

    ///
    /// Returns the frontend compiler version.
    ///
    fn version(&self) -> &solx_standard_json::Version;
}
