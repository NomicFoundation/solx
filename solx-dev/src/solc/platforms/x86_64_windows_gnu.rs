//!
//! `solx` solc x86_64 Windows GNU (MSYS2/MinGW) builder.
//!

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use crate::build_type::BuildType;
use crate::solc::boost::BoostConfig;
use crate::solc::platforms::shared;

/// Stack size for Windows (64MB).
const WINDOWS_STACK_SIZE: u64 = 67108864;

///
/// The building sequence for x86_64 Windows GNU.
///
/// Note: Windows requires local static Boost for proper static linking.
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
    use_gcc: bool,
) -> anyhow::Result<()> {
    // Windows requires local boost for static linking
    let boost_config = boost_config.ok_or_else(|| {
        anyhow::anyhow!(
            "Windows requires local Boost for static linking. Use --build-boost to build it."
        )
    })?;

    crate::utils::exists("cmake")?;
    crate::utils::exists("ninja")?;

    // Fix Boost library names for Windows (remove version suffix)
    fix_boost_library_names(boost_config)?;

    let mut cmake = Command::new("cmake");
    cmake.current_dir(build_dir);
    cmake.arg(source_dir);
    cmake.arg("-G").arg("Ninja");

    // Compiler selection
    if !use_gcc {
        crate::utils::exists("clang")?;
        crate::utils::exists("clang++")?;
        cmake.arg("-DCMAKE_C_COMPILER=clang");
        cmake.arg("-DCMAKE_CXX_COMPILER=clang++");
    }
    cmake.arg("-DUSE_LD_GOLD=OFF");

    // Windows-specific linker flags
    let ldflags =
        format!("-fuse-ld=lld -lbcrypt -lwsock32 -static -Wl,--stack,{WINDOWS_STACK_SIZE}");
    cmake.env("LDFLAGS", &ldflags);

    // Shared options
    for arg in shared::shared_cmake_args(build_type, pedantic, tests) {
        cmake.arg(arg);
    }

    // CXX flags
    cmake.arg(format!("-DCMAKE_CXX_FLAGS={}", shared::shared_cxx_flags()));

    // Boost configuration with Windows-specific adjustments
    let boost_root = boost_config.cmake_root();
    let boost_lib_dir = boost_config.lib_dir();
    let boost_include_dir = boost_config.windows_include_dir();
    for arg in shared::boost_cmake_args(&boost_root, &boost_lib_dir, &boost_include_dir) {
        cmake.arg(arg);
    }
    // Windows-specific Boost flags
    cmake.arg("-DBoost_COMPILER=-mgw15");
    cmake.arg("-DBoost_ARCHITECTURE=-x64");

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

///
/// Fix Boost library names for Windows.
///
/// The Boost build creates versioned library names like `libboost_system-mgw15-...`.
/// CMake expects them without the version suffix, so we create copies.
///
fn fix_boost_library_names(boost_config: &BoostConfig) -> anyhow::Result<()> {
    let lib_dir = boost_config.lib_dir();

    let libraries = [
        "libboost_system",
        "libboost_program_options",
        "libboost_filesystem",
        "libboost_thread",
        "libboost_date_time",
        "libboost_regex",
        "libboost_chrono",
        "libboost_random",
        "libboost_unit_test_framework",
    ];

    for lib_base in libraries {
        // Find the versioned library file
        let entries: Vec<PathBuf> = std::fs::read_dir(&lib_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with(lib_base) && name.ends_with(".a") && name.contains('-')
                    })
            })
            .collect();

        if let Some(versioned_lib) = entries.first() {
            let target_lib = lib_dir.join(format!("{lib_base}.a"));
            if !target_lib.exists() {
                std::fs::copy(versioned_lib, &target_lib)?;
                eprintln!(
                    "Created Boost library symlink: {} -> {}",
                    versioned_lib.display(),
                    target_lib.display()
                );
            }
        }
    }

    Ok(())
}
