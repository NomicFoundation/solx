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

use anyhow::Context;

///
/// Executes the building of the LLVM framework for the platform determined by the cfg macro.
/// Since cfg is evaluated at compile time, overriding the platform with a command-line
/// argument is not possible. So for cross-platform testing, comment out all but the
/// line to be tested, and perhaps also checks in the platform-specific build method.
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
                enable_mlir,
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
                enable_mlir,
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
                enable_mlir,
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
                enable_mlir,
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
                enable_mlir,
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

    if enable_mlir {
        create_mlir_link_stub()?;
    }

    Ok(())
}

/// Create an empty `libMLIR.a` archive in the LLVM install prefix.
///
/// mlir-sys unconditionally emits `cargo:rustc-link-lib=MLIR` expecting a
/// monolithic library, but the LLVM/MLIR build only produces individual
/// component libraries (all already linked by mlir-sys). This empty archive
/// satisfies the linker without duplicating symbols.
///
/// Fragility: if a future mlir-sys version expects symbols from the
/// monolithic libMLIR.a that are not in the component libraries, the link
/// will fail. Also, this stub is only created by `solx-dev llvm build
/// --enable-mlir`; manual LLVM builds must create it themselves.
fn create_mlir_link_stub() -> anyhow::Result<()> {
    let lib_dir = Path::llvm_target_final()?.join("lib");
    let stub_path = lib_dir.join("libMLIR.a");
    if !stub_path.exists() {
        std::fs::write(&stub_path, b"!<arch>\n")
            .with_context(|| format!("Failed to write MLIR stub archive to {stub_path:?}"))?;
    }
    Ok(())
}
