//!
//! `solx` LLVM tools.
//!

pub mod path;
pub mod platforms;
pub mod sanitizer;

pub use self::path::Path;
pub use self::sanitizer::Sanitizer;
pub use crate::build_type::BuildType;

use crate::ccache_variant::CcacheVariant;

///
/// Executes the building of the LLVM framework for the platform determined by the cfg macro.
/// Since cfg is evaluated at compile time, overriding the platform with a command-line
/// argument is not possible. So for cross-platform testing, comment out all but the
/// line to be tested, and perhaps also checks in the platform-specific build method.
///
pub fn build(
    build_type: BuildType,
    enable_utils: bool,
    install_distribution: bool,
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

    // LLVM's CMake bails with a fatal error when `LLVM_INCLUDE_TESTS=On` and
    // `LLVM_INCLUDE_UTILS=Off` (solx-llvm/llvm/CMakeLists.txt:1361, "Including
    // tests when not building utils will not work"), so promote `enable_tests`
    // to imply `enable_utils` rather than push that contract onto every caller.
    let enable_utils = enable_utils || enable_tests;
    // `--enable-tests` requires the full toolset (the regression suite
    // depends on every LLVM binary), so it overrides `--install-distribution`.
    let install_distribution = install_distribution && !enable_tests;

    if cfg!(target_arch = "x86_64") {
        if cfg!(target_os = "linux") {
            platforms::x86_64_linux_gnu::build(
                build_type,
                enable_utils,
                install_distribution,
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
                enable_utils,
                install_distribution,
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
                enable_utils,
                install_distribution,
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
                enable_utils,
                install_distribution,
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
                enable_utils,
                install_distribution,
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
