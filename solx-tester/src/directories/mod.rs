//!
//! The buildable compiler test trait.
//!

pub mod ethereum;
pub mod matter_labs;

use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

use crate::compilers::Compiler;
use crate::compilers::mode::Mode;
use crate::filters::Filters;
use crate::summary::Summary;
use crate::test::Test;

///
/// The compiler tests directory trait.
///
pub trait Collection {
    ///
    /// The test type.
    ///
    type Test: Buildable + std::fmt::Debug;

    ///
    /// Returns all directory tests.
    ///
    fn read_all(
        directory_path: &Path,
        extension: &'static str,
        summary: Arc<Mutex<Summary>>,
        filters: &Filters,
    ) -> anyhow::Result<Vec<Self::Test>>;
}

///
/// The buildable compiler test trait.
///
pub trait Buildable: Send + Sync + 'static {
    ///
    /// Builds the test for EVM.
    ///
    fn build_for_evm(
        &self,
        mode: Mode,
        compiler: Arc<dyn Compiler>,
        summary: Arc<Mutex<Summary>>,
        filters: &Filters,
        debug_config: Option<solx_codegen_evm::DebugConfig>,
    ) -> Option<Test>;
}
