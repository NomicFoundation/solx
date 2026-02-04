//!
//! `solx` solc aarch64 `linux-gnu` builder.
//!

use std::path::Path;
use std::process::Command;

use crate::solc::boost::BoostConfig;
use crate::solc::platforms::shared;

///
/// The building sequence for aarch64 Linux GNU.
///
pub fn build(
    source_dir: &Path,
    build_dir: &Path,
    build_type: &str,
    pedantic: bool,
    tests: bool,
    extra_args: Vec<String>,
    boost_config: Option<&BoostConfig>,
    enable_mlir: bool,
    use_gcc: bool,
) -> anyhow::Result<()> {
    crate::utils::exists("cmake")?;
    crate::utils::exists("ninja")?;

    let mut cmake = Command::new("cmake");
    cmake.current_dir(build_dir);
    cmake.arg(source_dir);
    cmake.arg("-G").arg("Ninja");

    // Compiler selection
    if !use_gcc {
        crate::utils::exists("clang")?;
        crate::utils::exists("clang++")?;
        crate::utils::exists("lld")?;
        cmake.arg("-DCMAKE_C_COMPILER=clang");
        cmake.arg("-DCMAKE_CXX_COMPILER=clang++");
        cmake.arg("-DUSE_LD_GOLD=OFF");
        cmake.env("LDFLAGS", "-fuse-ld=lld");
    } else {
        cmake.arg("-DUSE_LD_GOLD=OFF");
    }

    // Shared options
    for arg in shared::shared_cmake_args(build_type, pedantic, tests) {
        cmake.arg(arg);
    }

    // CXX flags
    cmake.arg(format!("-DCMAKE_CXX_FLAGS={}", shared::shared_cxx_flags()));

    // Boost configuration (only if local boost is available)
    if let Some(boost_config) = boost_config {
        let boost_root = boost_config.cmake_root();
        let boost_lib_dir = boost_config.lib_dir();
        let boost_include_dir = boost_config.include_dir();
        for arg in shared::boost_cmake_args(&boost_root, &boost_lib_dir, &boost_include_dir) {
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
    build_cmd.arg(build_type);
    build_cmd.arg("--parallel");
    build_cmd.arg(job_count.to_string());
    build_cmd.arg("--verbose");

    crate::utils::command(&mut build_cmd, "solc cmake build")?;

    Ok(())
}
