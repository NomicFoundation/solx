//!
//! `solx` LLVM tools.
//!

pub mod build_type;
pub mod ccache_variant;
pub mod path;
pub mod platforms;
pub mod project;
pub mod sanitizer;

pub use self::build_type::BuildType;
pub use self::ccache_variant::CcacheVariant;
pub use self::path::Path;
pub use self::project::Project;
pub use self::sanitizer::Sanitizer;

use std::collections::HashSet;

///
/// Executes the building of the LLVM framework for the platform determined by the cfg macro.
/// Since cfg is evaluated at compile time, overriding the platform with a command-line
/// argument is not possible. So for cross-platform testing, comment out all but the
/// line to be tested, and perhaps also checks in the platform-specific build method.
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
    enable_valgrind: bool,
    valgrind_options: Vec<String>,
    clean: bool,
) -> anyhow::Result<()> {
    if clean {
        let target_dir = std::path::PathBuf::from(Path::DIRECTORY_LLVM_TARGET);
        if target_dir.exists() {
            println!("Cleaning LLVM build directory: {}", target_dir.display());
            std::fs::remove_dir_all(&target_dir)?;
        }
    }

    std::fs::create_dir_all(Path::DIRECTORY_LLVM_TARGET)?;

    if cfg!(target_arch = "x86_64") {
        if cfg!(target_os = "linux") {
            platforms::x86_64_linux_gnu::build(
                build_type,
                llvm_projects,
                enable_rtti,
                enable_tests,
                enable_coverage,
                extra_args,
                ccache_variant,
                enable_assertions,
                sanitizer,
                enable_valgrind,
                valgrind_options,
            )?;
        } else if cfg!(target_os = "macos") {
            platforms::x86_64_macos::build(
                build_type,
                llvm_projects,
                enable_rtti,
                enable_tests,
                enable_coverage,
                extra_args,
                ccache_variant,
                enable_assertions,
                sanitizer,
            )?;
        } else if cfg!(target_os = "windows") {
            platforms::x86_64_windows_gnu::build(
                build_type,
                llvm_projects,
                enable_rtti,
                enable_tests,
                enable_coverage,
                extra_args,
                ccache_variant,
                enable_assertions,
                sanitizer,
            )?;
        } else {
            anyhow::bail!("Unsupported target OS for x86_64");
        }
    } else if cfg!(target_arch = "aarch64") {
        if cfg!(target_os = "linux") {
            platforms::aarch64_linux_gnu::build(
                build_type,
                llvm_projects,
                enable_rtti,
                enable_tests,
                enable_coverage,
                extra_args,
                ccache_variant,
                enable_assertions,
                sanitizer,
                enable_valgrind,
                valgrind_options,
            )?;
        } else if cfg!(target_os = "macos") {
            platforms::aarch64_macos::build(
                build_type,
                llvm_projects,
                enable_rtti,
                enable_tests,
                enable_coverage,
                extra_args,
                ccache_variant,
                enable_assertions,
                sanitizer,
            )?;
        } else {
            anyhow::bail!("Unsupported target OS for aarch64");
        }
    } else {
        anyhow::bail!("Unsupported target architecture");
    }

    Ok(())
}
