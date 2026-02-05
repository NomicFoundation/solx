//!
//! `solx` solc build tools.
//!

pub mod boost;
pub mod platforms;

use std::path::PathBuf;

use crate::build_type::BuildType;
use crate::solc::boost::BoostConfig;

/// The solc-solidity submodule directory.
pub const SOLIDITY_DIR: &str = "solx-solidity";

/// The build directory name.
pub const BUILD_DIR: &str = "build";

/// The LLVM build directory (where cmake configs are located).
pub const LLVM_BUILD_DIR: &str = "target-llvm/build-final";

///
/// Builds the solc libraries using cmake.
///
/// This function dispatches to platform-specific build implementations based on
/// the target architecture and OS (determined at compile time via cfg macros).
///
pub fn build(
    build_type: BuildType,
    pedantic: bool,
    tests: bool,
    extra_args: Vec<String>,
    clean: bool,
    build_boost: Option<String>,
    enable_mlir: bool,
    use_gcc: bool,
    use_ccache: bool,
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

    // Boost configuration: if --build-boost is provided, build local boost;
    // otherwise use system boost
    let boost_config = if let Some(version) = build_boost {
        let boost_base_dir = solidity_dir.join("boost");
        let boost_config = BoostConfig::new(version.clone(), boost_base_dir);

        // Check if already built
        if boost_config.lib_dir().exists() {
            let canonical_base_dir = boost_config.base_dir.canonicalize()?;
            eprintln!("Using existing Boost at {}", canonical_base_dir.display());
            Some(BoostConfig::new(version, canonical_base_dir))
        } else {
            // Download and build boost
            let install_path = boost::download_and_build(&solidity_dir, &boost_config)?;
            Some(BoostConfig::new(version, install_path))
        }
    } else {
        // No --build-boost provided, use system boost
        eprintln!("No --build-boost specified. Using system Boost.");
        None
    };

    // Canonicalize paths for cmake
    let source_dir = normalize_path(&solidity_dir.canonicalize()?);
    let build_dir_canonical =
        normalize_path(&build_dir.canonicalize().unwrap_or(build_dir.clone()));

    // Dispatch to platform-specific builder
    if cfg!(target_arch = "x86_64") {
        if cfg!(target_os = "linux") {
            platforms::x86_64_linux_gnu::build(
                &source_dir,
                &build_dir_canonical,
                build_type,
                pedantic,
                tests,
                extra_args,
                boost_config.as_ref(),
                enable_mlir,
                use_gcc,
                use_ccache,
            )?;
        } else if cfg!(target_os = "macos") {
            platforms::x86_64_macos::build(
                &source_dir,
                &build_dir_canonical,
                build_type,
                pedantic,
                tests,
                extra_args,
                boost_config.as_ref(),
                enable_mlir,
                use_ccache,
            )?;
        } else if cfg!(target_os = "windows") {
            platforms::x86_64_windows_gnu::build(
                &source_dir,
                &build_dir_canonical,
                build_type,
                pedantic,
                tests,
                extra_args,
                boost_config.as_ref(),
                enable_mlir,
                use_gcc,
                use_ccache,
            )?;
        } else {
            anyhow::bail!("Unsupported target OS for x86_64");
        }
    } else if cfg!(target_arch = "aarch64") {
        if cfg!(target_os = "linux") {
            platforms::aarch64_linux_gnu::build(
                &source_dir,
                &build_dir_canonical,
                build_type,
                pedantic,
                tests,
                extra_args,
                boost_config.as_ref(),
                enable_mlir,
                use_gcc,
                use_ccache,
            )?;
        } else if cfg!(target_os = "macos") {
            platforms::aarch64_macos::build(
                &source_dir,
                &build_dir_canonical,
                build_type,
                pedantic,
                tests,
                extra_args,
                boost_config.as_ref(),
                enable_mlir,
                use_ccache,
            )?;
        } else {
            anyhow::bail!("Unsupported target OS for aarch64");
        }
    } else {
        anyhow::bail!("Unsupported target architecture");
    }

    println!(
        "solc libraries built successfully in: {}",
        build_dir.display()
    );
    Ok(())
}

///
/// Normalizes a path by removing Windows extended-length prefix.
///
/// On Windows, `canonicalize()` returns paths with `\\?\` prefix which
/// can cause issues with some tools (e.g., git submodule operations).
///
fn normalize_path(path: &std::path::Path) -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        let path_str = path.display().to_string();
        if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }

    path.to_path_buf()
}
