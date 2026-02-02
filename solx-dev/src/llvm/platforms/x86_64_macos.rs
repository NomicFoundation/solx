//!
//! `solx` LLVM amd64 `macos` builder.
//!

use std::collections::HashSet;
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
    crate::utils::exists("ninja")?;

    let llvm_module_llvm = Path::llvm_module_llvm()?;
    let llvm_build_final = Path::llvm_build_final()?;
    let llvm_target_final = Path::llvm_target_final()?;

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
                format!(
                    "-DLLVM_ENABLE_PROJECTS='{}'",
                    llvm_projects
                        .into_iter()
                        .map(|project| project.to_string())
                        .collect::<Vec<String>>()
                        .join(";")
                )
                .as_str(),
                "-DCMAKE_OSX_DEPLOYMENT_TARGET='11.0'",
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
            .args(crate::llvm::platforms::shared::macos_build_opts_ignore_dupicate_libs_warnings())
            .args(crate::llvm::platforms::shared::shared_build_opts_sanitizers(sanitizer)),
        "LLVM building cmake",
    )?;

    crate::utils::ninja(llvm_build_final.as_ref())?;

    Ok(())
}
