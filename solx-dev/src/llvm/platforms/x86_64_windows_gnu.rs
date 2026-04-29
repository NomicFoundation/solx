//!
//! `solx` LLVM amd64 `windows-gnu` builder.
//!

use std::path::PathBuf;
use std::process::Command;

use crate::build_type::BuildType;
use crate::ccache_variant::CcacheVariant;
use crate::llvm::path::Path;
use crate::llvm::sanitizer::Sanitizer;

///
/// The building sequence.
///
pub fn build(
    build_type: BuildType,
    enable_mlir: bool,
    enable_tests: bool,
    enable_coverage: bool,
    extra_args: Vec<String>,
    ccache_variant: Option<CcacheVariant>,
    enable_assertions: bool,
    sanitizer: Option<Sanitizer>,
) -> anyhow::Result<()> {
    crate::utils::exists("cmake")?;
    crate::utils::exists("clang")?;
    crate::utils::exists("clang++")?;
    crate::utils::exists("lld")?;
    crate::utils::exists("ninja")?;

    let llvm_module_llvm = Path::llvm_module_llvm().and_then(crate::utils::path_windows_to_unix)?;
    let llvm_build_final = Path::llvm_build_final().and_then(crate::utils::path_windows_to_unix)?;
    let llvm_target_final =
        Path::llvm_target_final().and_then(crate::utils::path_windows_to_unix)?;

    let llvm_module_llvm_str = llvm_module_llvm.to_string_lossy();
    let llvm_build_final_str = llvm_build_final.to_string_lossy();
    let llvm_target_final_str = llvm_target_final.to_string_lossy();

    crate::utils::command(
        Command::new("cmake")
            .args([
                "-S",
                &*llvm_module_llvm_str,
                "-B",
                &*llvm_build_final_str,
                "-G",
                "Ninja",
                format!("-DCMAKE_INSTALL_PREFIX='{llvm_target_final_str}'",).as_str(),
                format!("-DCMAKE_BUILD_TYPE='{build_type}'").as_str(),
                "-DCMAKE_C_COMPILER='clang'",
                "-DCMAKE_CXX_COMPILER='clang++'",
                "-DLLVM_USE_LINKER='lld'",
            ])
            .args(crate::llvm::platforms::shared::shared_build_opts_projects(
                enable_mlir,
            ))
            .args(crate::llvm::platforms::shared::shared_build_opts_targets())
            .args(crate::llvm::platforms::shared::shared_build_opts_tests(
                enable_tests,
            ))
            .args(crate::llvm::platforms::shared::shared_build_opts_coverage(
                enable_coverage,
            ))
            .args(crate::llvm::platforms::shared::SHARED_BUILD_OPTS)
            .args(crate::llvm::platforms::shared::shared_build_opts_werror())
            .args(crate::llvm::platforms::shared::windows_build_opts_distribution(enable_mlir))
            .args(extra_args)
            .args(CcacheVariant::cmake_args(ccache_variant))
            .args(crate::llvm::platforms::shared::shared_build_opts_assertions(enable_assertions))
            .args(crate::llvm::platforms::shared::shared_build_opts_sanitizers(sanitizer)),
        "LLVM building cmake",
    )?;

    // Windows uses `install-distribution` + an explicit LLVM_DISTRIBUTION_COMPONENTS
    // whitelist (set by `windows_build_opts_distribution`) to skip installing the
    // ~200 LLVM tool binaries solx doesn't use. See #364.
    crate::utils::ninja(llvm_build_final.as_ref(), "install-distribution")?;

    // `llvm-config` is excluded from the distribution because LLVM_BUILD_TOOLS=Off
    // prevents its install target from ever being defined. Build and copy it by
    // hand so llvm-sys can find it at Rust build time.
    crate::utils::command(
        Command::new("ninja")
            .arg("-C")
            .arg(&*llvm_build_final_str)
            .arg("llvm-config"),
        "Building llvm-config",
    )?;
    let llvm_config_source = llvm_build_final.join("bin").join("llvm-config.exe");
    let llvm_config_dest_dir = llvm_target_final.join("bin");
    std::fs::create_dir_all(&llvm_config_dest_dir)?;
    fs_extra::file::copy(
        crate::utils::path_windows_to_unix(llvm_config_source)?,
        crate::utils::path_windows_to_unix(llvm_config_dest_dir.join("llvm-config.exe"))?,
        &fs_extra::file::CopyOptions {
            overwrite: true,
            ..Default::default()
        },
    )?;

    let libstdcpp_source_path = match std::env::var("LIBSTDCPP_SOURCE_PATH") {
        Ok(libstdcpp_source_path) => PathBuf::from(libstdcpp_source_path),
        Err(error) => anyhow::bail!(
            "The `LIBSTDCPP_SOURCE_PATH` must be set to the path to the libstdc++.a static library: {error}"
        ),
    };
    let mut libstdcpp_destination_path = llvm_target_final;
    libstdcpp_destination_path.push("./lib/libstdc++.a");
    fs_extra::file::copy(
        crate::utils::path_windows_to_unix(libstdcpp_source_path)?,
        crate::utils::path_windows_to_unix(libstdcpp_destination_path)?,
        &fs_extra::file::CopyOptions {
            overwrite: true,
            ..Default::default()
        },
    )?;

    Ok(())
}
