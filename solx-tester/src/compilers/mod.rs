//!
//! The contract compilers for different languages.
//!

pub mod cache;
pub mod llvm_ir;
pub mod mode;
pub mod solidity;
pub mod yul;

use crate::revm::input::Input as EVMInput;

use self::mode::Mode;

///
/// The compiler trait.
///
pub trait Compiler: Send + Sync + 'static {
    ///
    /// Compile all sources for EVM.
    ///
    fn compile_for_evm(
        &self,
        test_path: String,
        sources: Vec<(String, String)>,
        libraries: solx_utils::Libraries,
        mode: &Mode,
        test_params: Option<&solx_solc_test_adapter::Params>,
        llvm_options: Vec<String>,
        debug_config: Option<solx_codegen_evm::OutputConfig>,
    ) -> anyhow::Result<EVMInput>;

    ///
    /// Returns all supported combinations of compiler settings.
    ///
    fn all_modes(&self) -> Vec<Mode>;

    ///
    /// Whether one source file can contains multiple contracts.
    ///
    fn allows_multi_contract_files(&self) -> bool;
}
