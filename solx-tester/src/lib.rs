//!
//! `solx` tester library.
//!

#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

pub(crate) mod compilers;
pub(crate) mod directories;
pub(crate) mod filters;
pub(crate) mod revm;
pub(crate) mod summary;
pub(crate) mod test;
pub(crate) mod toolchain;
pub(crate) mod utils;
pub(crate) mod workflow;

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

use itertools::Itertools;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

pub use crate::compilers::Compiler;
pub use crate::compilers::llvm_ir::LLVMIRCompiler;
pub use crate::compilers::mode::Mode;
pub use crate::compilers::mode::llvm_options::LLVMOptions;
pub use crate::compilers::solidity::solc::SolidityCompiler as SolcCompiler;
pub use crate::compilers::solidity::solx::SolidityCompiler as SolxCompiler;
pub use crate::compilers::yul::YulCompiler;
pub use crate::directories::Buildable;
pub use crate::directories::Collection;
pub use crate::directories::ethereum::EthereumDirectory;
pub use crate::directories::ethereum::test::EthereumTest;
pub use crate::directories::matter_labs::MatterLabsDirectory;
pub use crate::filters::Filters;
pub use crate::revm::REVM;
pub use crate::summary::Summary;
pub use crate::toolchain::Toolchain;
pub use crate::workflow::Workflow;

/// The debug directory path.
pub const DEBUG_DIRECTORY: &str = "./debug/";

///
/// The compiler test generic representation.
///
type Test = (Arc<dyn Buildable>, Arc<dyn Compiler>, Mode);

///
/// `solx` tester.
///
pub struct SolxTester<'a> {
    /// The summary.
    pub summary: Arc<Mutex<Summary>>,
    /// The filters.
    pub filters: Filters<'a>,
    /// The debug config.
    pub debug_config: Option<solx_codegen_evm::DebugConfig>,
    /// Actions to perform.
    pub workflow: Workflow,
}

impl<'a> SolxTester<'a> {
    /// The Solidity simple tests directory.
    const SOLIDITY_SIMPLE: &'static str = "tests/solidity/simple";
    /// The Solidity complex tests directory.
    const SOLIDITY_COMPLEX: &'static str = "tests/solidity/complex";
    /// The Solidity upstream tests directory.
    const SOLIDITY_UPSTREAM: &'static str = "solx-solidity/test/libsolidity/semanticTests";

    /// The Yul simple tests directory.
    const YUL_SIMPLE: &'static str = "tests/yul";

    /// The LLVM IR simple tests directory.
    const LLVM_IR_SIMPLE: &'static str = "tests/llvm-ir";
}

impl<'a> SolxTester<'a> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        summary: Arc<Mutex<Summary>>,
        filters: Filters<'a>,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
        workflow: Workflow,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            summary,
            filters,
            debug_config,
            workflow,
        })
    }

    ///
    /// Runs all tests on REVM.
    ///
    pub fn run_revm(
        self,
        toolchain: Toolchain,
        solx: Option<PathBuf>,
        enable_trace: bool,
    ) -> anyhow::Result<()> {
        let tests = self.all_tests(toolchain, solx)?;

        let _: Vec<()> = tests
            .into_par_iter()
            .map(|(test, compiler, mode)| {
                let mode_string = mode.to_string();
                let specialized_debug_config = self
                    .debug_config
                    .as_ref()
                    .and_then(|config| config.create_subdirectory(mode_string.as_str()).ok());
                if let Some(test) = test.build_for_evm(
                    mode,
                    compiler,
                    self.summary.clone(),
                    &self.filters,
                    specialized_debug_config,
                ) && let Workflow::BuildAndRun = self.workflow
                {
                    test.run_revm(self.summary.clone(), enable_trace)
                };
            })
            .collect();

        Ok(())
    }

    ///
    /// Returns all tests from all directories.
    ///
    fn all_tests(&self, toolchain: Toolchain, solx: Option<PathBuf>) -> anyhow::Result<Vec<Test>> {
        let solx_path = solx.unwrap_or_else(|| PathBuf::from("solx"));
        let solidity_compiler = Arc::new(SolxCompiler::try_from_path(solx_path)?);
        let llvm_ir_compiler = Arc::new(LLVMIRCompiler::Solx(solidity_compiler.clone()));

        let (solidity_compiler, yul_compiler, llvm_ir_compiler): (
            Arc<dyn Compiler>,
            Arc<dyn Compiler>,
            Arc<dyn Compiler>,
        ) = match toolchain {
            Toolchain::Solx => {
                let yul_compiler = Arc::new(YulCompiler::Solx(solidity_compiler.clone()));
                (solidity_compiler, yul_compiler, llvm_ir_compiler)
            }
            Toolchain::Solc | Toolchain::SolxMlir => {
                let solidity_compiler = Arc::new(SolcCompiler::new(
                    solx_standard_json::InputLanguage::Solidity,
                    toolchain,
                ));
                let yul_compiler = Arc::new(SolcCompiler::new(
                    solx_standard_json::InputLanguage::Yul,
                    toolchain,
                ));
                (solidity_compiler, yul_compiler, llvm_ir_compiler)
            }
        };

        let mut tests = Vec::with_capacity(16384);

        tests.extend(self.directory::<MatterLabsDirectory>(
            Self::SOLIDITY_SIMPLE,
            solx_utils::EXTENSION_SOLIDITY,
            solidity_compiler.clone(),
        )?);
        tests.extend(self.directory::<MatterLabsDirectory>(
            Self::SOLIDITY_COMPLEX,
            solx_utils::EXTENSION_JSON,
            solidity_compiler.clone(),
        )?);
        tests.extend(self.directory::<EthereumDirectory>(
            Self::SOLIDITY_UPSTREAM,
            solx_utils::EXTENSION_SOLIDITY,
            solidity_compiler.clone(),
        )?);

        tests.extend(self.directory::<MatterLabsDirectory>(
            Self::YUL_SIMPLE,
            solx_utils::EXTENSION_YUL,
            yul_compiler,
        )?);

        tests.extend(self.directory::<MatterLabsDirectory>(
            Self::LLVM_IR_SIMPLE,
            solx_utils::EXTENSION_LLVM_SOURCE,
            llvm_ir_compiler,
        )?);

        Ok(tests)
    }

    ///
    /// Returns all tests from the specified directory for the specified compiler.
    ///
    fn directory<T>(
        &self,
        path: &str,
        extension: &'static str,
        compiler: Arc<dyn Compiler>,
    ) -> anyhow::Result<Vec<Test>>
    where
        T: Collection,
    {
        let directory_path = crate::utils::str_to_path_normalized(path);
        Ok(T::read_all(
            directory_path.as_path(),
            extension,
            self.summary.clone(),
            &self.filters,
        )
        .map_err(|error| {
            anyhow::anyhow!("Failed to read the tests directory {directory_path:?}: {error}")
        })?
        .into_iter()
        .map(|test| Arc::new(test) as Arc<dyn Buildable>)
        .cartesian_product(compiler.all_modes())
        .map(|(test, mode)| (test, compiler.clone() as Arc<dyn Compiler>, mode))
        .collect())
    }
}
