//!
//! The contract compilers for different languages.
//!

pub mod cache;
pub mod input_ext;
pub mod llvm_ir;
pub mod mode;
pub mod output_ext;
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

///
/// Returns all possible combinations of the optimizer settings.
///
/// Every combination uses the aggressive back-end level to maximize
/// optimization pressure across all middle-end levels.
///
pub fn optimizer_combinations() -> Vec<solx_codegen_evm::OptimizerSettings> {
    let performance = [
        solx_codegen_evm::OptimizerSettings::try_from_cli('1').expect("Always valid"),
        solx_codegen_evm::OptimizerSettings::try_from_cli('2').expect("Always valid"),
        solx_codegen_evm::OptimizerSettings::try_from_cli('3').expect("Always valid"),
    ];
    let size = [
        solx_codegen_evm::OptimizerSettings::try_from_cli('s').expect("Always valid"),
        solx_codegen_evm::OptimizerSettings::try_from_cli('z').expect("Always valid"),
    ];

    // Override back-end to Aggressive for performance levels to create more
    // variance than the CLI presets (which use matching back-end levels).
    let aggressive_back_end = solx_codegen_evm::OptimizerSettings::cycles().level_back_end;
    performance
        .into_iter()
        .map(|mut settings| {
            settings.level_back_end = aggressive_back_end;
            settings
        })
        .chain(size)
        .collect()
}
