//!
//! `solx` solc x86_64 macOS builder.
//!

use std::path::Path;
use std::process::Command;

use crate::build_type::BuildType;
use crate::solc::boost::BoostConfig;
use crate::solc::platforms::shared;

///
/// The building sequence for x86_64 macOS.
///
pub fn build(
    source_dir: &Path,
    build_dir: &Path,
    build_type: BuildType,
    pedantic: bool,
    tests: bool,
    extra_args: Vec<String>,
    boost_config: Option<&BoostConfig>,
    enable_mlir: bool,
) -> anyhow::Result<()> {
    crate::utils::exists("cmake")?;
    let ninja_available = crate::utils::exists("ninja").is_ok();

    let mut cmake = Command::new("cmake");
    cmake.current_dir(build_dir);
    cmake.arg(source_dir);
    if ninja_available {
        cmake.arg("-G").arg("Ninja");
    }

    // Shared options
    for arg in shared::shared_cmake_args(build_type, pedantic, tests) {
        cmake.arg(arg);
    }

    // CXX flags
    cmake.arg(format!("-DCMAKE_CXX_FLAGS={}", shared::shared_cxx_flags()));

    // Handle duplicate libs warnings for XCode 15+
    let xcode_version =
        crate::utils::get_xcode_version().unwrap_or(crate::utils::XCODE_MIN_VERSION);
    if xcode_version >= crate::utils::XCODE_VERSION_15 {
        cmake.arg("-DCMAKE_EXE_LINKER_FLAGS=-Wl,-no_warn_duplicate_libraries");
        cmake.arg("-DCMAKE_SHARED_LINKER_FLAGS=-Wl,-no_warn_duplicate_libraries");
    }

    // Boost configuration (only if local boost is available)
    if let Some(boost_config) = boost_config {
        let boost_lib_dir = boost_config.lib_dir();
        let boost_include_dir = boost_config.include_dir();
        for arg in
            shared::boost_cmake_args(&boost_config.version, &boost_lib_dir, &boost_include_dir)
        {
            cmake.arg(arg);
        }
    }

    // MLIR configuration
    if enable_mlir {
        let llvm_build_dir = std::path::PathBuf::from(crate::solc::LLVM_BUILD_DIR);
        for arg in shared::mlir_cmake_args(&llvm_build_dir) {
            cmake.arg(arg);
        }
    }

    // Extra arguments
    for arg in extra_args {
        cmake.arg(arg);
    }

    crate::utils::command(&mut cmake, "solc cmake configure")?;

    // Build
    let job_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);

    let mut build_cmd = Command::new("cmake");
    build_cmd.arg("--build");
    build_cmd.arg(build_dir);
    build_cmd.arg("--config");
    build_cmd.arg(build_type.to_string());
    build_cmd.arg("--parallel");
    build_cmd.arg(job_count.to_string());
    build_cmd.arg("--verbose");

    crate::utils::command(&mut build_cmd, "solc cmake build")?;

    Ok(())
}
