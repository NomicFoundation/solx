//!
//! `solx` LLVM tool arguments.
//!

use clap::Args;

///
/// `solx` LLVM tool arguments.
///
#[derive(Args)]
pub struct Build {
    /// Enable verbose output, e.g. to inspect extra flags.
    #[arg(short, long)]
    pub verbose: bool,

    //
    // Build Configuration
    //
    /// LLVM build type (`Debug`, `Release`, `RelWithDebInfo`, or `MinSizeRel`).
    #[arg(long, default_value_t = solx_dev::LLVMBuildType::Release, help_heading = "Build Configuration")]
    pub build_type: solx_dev::LLVMBuildType,

    /// Clean the build directory before building.
    #[arg(long)]
    pub clean: bool,

    /// LLVM projects to build LLVM with.
    #[arg(long, num_args = 1.., help_heading = "Build Configuration")]
    pub llvm_projects: Vec<solx_dev::LLVMProject>,

    /// Extra arguments to pass to CMake.
    /// A leading backslash will be unescaped.
    #[arg(long, num_args = 1.., help_heading = "Build Configuration")]
    pub extra_args: Vec<String>,

    /// Whether to use compiler cache (ccache) to speed-up builds.
    #[arg(long, help_heading = "Build Configuration")]
    pub ccache_variant: Option<solx_dev::LLVMCcacheVariant>,

    //
    // Build Features
    //
    /// Whether to build LLVM with run-time type information (RTTI) enabled.
    #[arg(long, help_heading = "Build Features")]
    pub enable_rtti: bool,

    /// Whether to build the LLVM tests.
    #[arg(long, help_heading = "Build Features")]
    pub enable_tests: bool,

    /// Whether to build LLVM for source-based code coverage.
    #[arg(long, help_heading = "Build Features")]
    pub enable_coverage: bool,

    /// Whether to build with assertions enabled or not.
    #[arg(long, help_heading = "Build Features")]
    pub enable_assertions: bool,

    //
    // Debugging & Sanitizers
    //
    /// Build LLVM with sanitizer enabled (`Address`, `Memory`, `MemoryWithOrigins`, `Undefined`, `Thread`, `DataFlow`, or `Address;Undefined`).
    #[arg(long, help_heading = "Debugging & Sanitizers")]
    pub sanitizer: Option<solx_dev::LLVMSanitizer>,

    /// Whether to run LLVM unit tests under valgrind or not.
    #[arg(long, help_heading = "Debugging & Sanitizers")]
    pub enable_valgrind: bool,

    /// Additional valgrind options to pass to the valgrind command.
    #[arg(long, help_heading = "Debugging & Sanitizers")]
    pub valgrind_options: Vec<String>,
}
