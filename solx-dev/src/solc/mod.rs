//!
//! `solx` solc build tools.
//!

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// The solc-solidity submodule directory.
pub const SOLIDITY_DIR: &str = "solx-solidity";

/// The build directory name.
pub const BUILD_DIR: &str = "build";

/// The LLVM build directory (where cmake configs are located).
pub const LLVM_BUILD_DIR: &str = "target-llvm/build-final";

///
/// Builds the solc libraries using cmake.
///
pub fn build(
    build_type: String,
    pedantic: bool,
    tests: bool,
    extra_args: Vec<String>,
    clean: bool,
) -> anyhow::Result<()> {
    let solidity_dir = PathBuf::from(SOLIDITY_DIR);
    if !solidity_dir.exists() {
        anyhow::bail!(
            "solx-solidity directory not found. Please run: git submodule update --recursive --checkout"
        );
    }

    let build_dir = solidity_dir.join(BUILD_DIR);

    if clean && build_dir.exists() {
        println!("Cleaning build directory: {}", build_dir.display());
        std::fs::remove_dir_all(&build_dir)?;
    }

    std::fs::create_dir_all(&build_dir)?;

    // Run cmake configure
    configure(
        &build_dir,
        &solidity_dir,
        &build_type,
        pedantic,
        tests,
        extra_args,
    )?;

    // Run cmake build
    build_cmake(&build_dir, &build_type)?;

    println!(
        "solc libraries built successfully in: {}",
        build_dir.display()
    );
    Ok(())
}

///
/// Runs cmake configure step.
///
fn configure(
    build_dir: &Path,
    source_dir: &Path,
    build_type: &str,
    pedantic: bool,
    tests: bool,
    extra_args: Vec<String>,
) -> anyhow::Result<()> {
    println!("Configuring solc build...");

    let mut command = Command::new("cmake");
    command.current_dir(build_dir);
    command.arg(source_dir.canonicalize()?);

    // Standard options
    command.arg(format!(
        "-DPEDANTIC={}",
        if pedantic { "ON" } else { "OFF" }
    ));
    command.arg(format!("-DTESTS={}", if tests { "ON" } else { "OFF" }));
    command.arg(format!("-DCMAKE_BUILD_TYPE={}", build_type));

    // MLIR/LLD paths (if LLVM was built with these projects)
    let llvm_build_dir = PathBuf::from(LLVM_BUILD_DIR);
    if llvm_build_dir.exists() {
        let mlir_dir = llvm_build_dir.join("lib/cmake/mlir");
        if mlir_dir.exists() {
            command.arg(format!("-DMLIR_DIR={}", mlir_dir.canonicalize()?.display()));
        }

        let lld_dir = llvm_build_dir.join("lib/cmake/lld");
        if lld_dir.exists() {
            command.arg(format!("-DLLD_DIR={}", lld_dir.canonicalize()?.display()));
        }
    }

    // Extra arguments
    for arg in extra_args {
        command.arg(arg);
    }

    println!(
        "Running: cmake {}",
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let status = command.status()?;
    if !status.success() {
        anyhow::bail!("cmake configure failed with status: {status}");
    }

    Ok(())
}

///
/// Runs cmake build step.
///
fn build_cmake(build_dir: &Path, build_type: &str) -> anyhow::Result<()> {
    println!("Building solc libraries...");

    let mut command = Command::new("cmake");
    command.arg("--build");
    command.arg(build_dir);
    command.arg("--config");
    command.arg(build_type);

    let job_count = std::thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1);
    command.arg("--parallel");
    command.arg(job_count.to_string());

    println!(
        "Running: cmake {}",
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let status = command.status()?;
    if !status.success() {
        anyhow::bail!("cmake build failed with status: {status}");
    }

    Ok(())
}
