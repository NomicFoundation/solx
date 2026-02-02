//!
//! `solx` LLVM amd64 `windows-gnu` builder.
//!

use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

use crate::llvm::build_type::BuildType;
use crate::llvm::ccache_variant::CcacheVariant;
use crate::llvm::path::Path;
use crate::llvm::project::Project;
use crate::llvm::sanitizer::Sanitizer;

///
/// The building sequence.
///
pub fn build(
    build_type: BuildType,
    llvm_projects: HashSet<Project>,
    enable_rtti: bool,
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
                format!(
                    "-DLLVM_ENABLE_PROJECTS='{}'",
                    llvm_projects
                        .into_iter()
                        .map(|project| project.to_string())
                        .collect::<Vec<String>>()
                        .join(";")
                )
                .as_str(),
                "-DLLVM_USE_LINKER='lld'",
            ])
            .args(crate::llvm::platforms::shared::shared_build_opts_targets())
            .args(crate::llvm::platforms::shared::shared_build_opts_tests(
                enable_tests,
            ))
            .args(crate::llvm::platforms::shared::shared_build_opts_coverage(
                enable_coverage,
            ))
            .args(crate::llvm::platforms::shared::SHARED_BUILD_OPTS)
            .args(crate::llvm::platforms::shared::shared_build_opts_werror())
            .args(extra_args)
            .args(crate::llvm::platforms::shared::shared_build_opts_ccache(
                ccache_variant,
            ))
            .args(crate::llvm::platforms::shared::shared_build_opts_assertions(enable_assertions))
            .args(crate::llvm::platforms::shared::shared_build_opts_rtti(
                enable_rtti,
            ))
            .args(crate::llvm::platforms::shared::shared_build_opts_sanitizers(sanitizer)),
        "LLVM building cmake",
    )?;

    crate::utils::ninja(llvm_build_final.as_ref())?;

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
        &fs_extra::file::CopyOptions::default(),
    )?;

    Ok(())
}
