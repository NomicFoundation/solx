//!
//! The shared options for building various platforms.
//!

use std::path::Path;
use std::process::Command;

use crate::build_type::BuildType;
use crate::llvm::platforms::Platform;
use crate::llvm::sanitizer::Sanitizer;

/// The build options shared by all platforms.
pub const SHARED_BUILD_OPTS: [&str; 23] = [
    "-DPACKAGE_VENDOR='Matter Labs'",
    "-DCMAKE_BUILD_WITH_INSTALL_RPATH=1",
    "-DLLVM_BUILD_DOCS='Off'",
    "-DLLVM_BUILD_RUNTIME='Off'",
    "-DLLVM_BUILD_RUNTIMES='Off'",
    "-DLLVM_INCLUDE_DOCS='Off'",
    "-DLLVM_INCLUDE_BENCHMARKS='Off'",
    "-DLLVM_INCLUDE_EXAMPLES='Off'",
    "-DLLVM_INCLUDE_RUNTIMES='Off'",
    "-DLLVM_ENABLE_RTTI='On'",
    "-DLLVM_ENABLE_DOXYGEN='Off'",
    "-DLLVM_ENABLE_SPHINX='Off'",
    "-DLLVM_ENABLE_OCAMLDOC='Off'",
    "-DLLVM_ENABLE_ZLIB='Off'",
    "-DLLVM_ENABLE_ZSTD='Off'",
    "-DLLVM_ENABLE_LIBXML2='Off'",
    "-DLLVM_ENABLE_BINDINGS='Off'",
    "-DLLVM_ENABLE_LIBEDIT='Off'",
    "-DLLVM_ENABLE_LIBPFM='Off'",
    "-DLLVM_OPTIMIZED_TABLEGEN='Off'",
    "-DCMAKE_EXPORT_COMPILE_COMMANDS='On'",
    "-DPython3_FIND_REGISTRY='LAST'", // Use Python version from $PATH, not from registry
    "-DBUG_REPORT_URL='https://github.com/matter-labs/solx-llvm/issues'",
];

///
/// The CMake argument for LLVM_ENABLE_PROJECTS.
/// LLD is always included; MLIR is added when `enable_mlir` is true.
///
pub fn shared_build_opts_projects(enable_mlir: bool) -> Vec<String> {
    let mut projects = vec!["lld"];
    if enable_mlir {
        projects.push("mlir");
    }
    vec![format!("-DLLVM_ENABLE_PROJECTS='{}'", projects.join(";"))]
}

///
/// The shared build options to treat warnings as errors.
///
/// Disabled on Windows due to the following upstream issue with MSYS2 with mingw-w64:
/// ProgramTest.cpp:23:15: error: '__p__environ' redeclared without 'dllimport' attribute
///
/// TODO: enable at least for non-Windows platforms
///
pub fn shared_build_opts_werror() -> Vec<String> {
    vec!["-DLLVM_ENABLE_WERROR='Off'".to_owned()]
}

///
/// The build options to enable assertions.
///
pub fn shared_build_opts_assertions(enabled: bool) -> Vec<String> {
    vec![format!(
        "-DLLVM_ENABLE_ASSERTIONS='{}'",
        if enabled { "On" } else { "Off" },
    )]
}

///
/// The build options to enable sanitizers.
///
pub fn shared_build_opts_sanitizers(sanitizer: Option<Sanitizer>) -> Vec<String> {
    match sanitizer {
        Some(sanitizer) => vec![format!("-DLLVM_USE_SANITIZER='{sanitizer}'")],
        None => vec![],
    }
}

///
/// The build options to enable split DWARF debug info on Linux.
///
/// Split DWARF keeps debug info in separate `.dwo` files so the linker never
/// processes it, significantly reducing link time. Only useful when debug info
/// is generated (Debug / RelWithDebInfo) and only supported on ELF targets
/// (Linux). Release and MinSizeRel builds keep full embedded DWARF.
///
pub fn shared_build_opts_split_dwarf(build_type: BuildType) -> Vec<String> {
    if cfg!(target_os = "linux")
        && matches!(build_type, BuildType::Debug | BuildType::RelWithDebInfo)
    {
        vec!["-DLLVM_USE_SPLIT_DWARF='On'".to_owned()]
    } else {
        vec![]
    }
}

///
/// The build options to enable Valgrind for LLVM regression tests.
///
pub fn shared_build_opts_valgrind(enabled: bool, valgrind_options: Vec<String>) -> Vec<String> {
    if !enabled {
        return vec![];
    }

    let vg_args = valgrind_options
        .iter()
        .map(|opt| format!("--vg-arg='{opt}'"))
        .collect::<Vec<_>>()
        .join(" ");

    vec![format!("-DLLVM_LIT_ARGS='-sv --vg --vg-leak {vg_args}'")]
}

///
/// The LLVM targets build options shared by all platforms.
///
pub fn shared_build_opts_targets() -> Vec<String> {
    vec![
        "-DLLVM_TARGETS_TO_BUILD=''".to_owned(),
        format!(
            "-DLLVM_EXPERIMENTAL_TARGETS_TO_BUILD='{}'",
            [Platform::EVM]
                .into_iter()
                .map(|platform| platform.to_string())
                .collect::<Vec<String>>()
                .join(";")
        ),
        format!("-DLLVM_DEFAULT_TARGET_TRIPLE='{}'", solx_utils::Target::EVM),
    ]
}

///
/// The LLVM utils build options shared by all platforms.
///
/// Toggles `FileCheck`, `llvm-lit`, and the rest of the LLVM utility binaries
/// (`-DLLVM_{BUILD,INCLUDE,INSTALL}_UTILS`). Sufficient for workflows that
/// only need the lit-test runners against pre-built MLIR/IR fixtures, without
/// pulling in the full `--enable-tests` build.
///
pub fn shared_build_opts_utils(enabled: bool) -> Vec<String> {
    vec![
        format!(
            "-DLLVM_BUILD_UTILS='{}'",
            if enabled { "On" } else { "Off" },
        ),
        format!(
            "-DLLVM_INCLUDE_UTILS='{}'",
            if enabled { "On" } else { "Off" },
        ),
        format!(
            "-DLLVM_INSTALL_UTILS='{}'",
            if enabled { "On" } else { "Off" },
        ),
    ]
}

///
/// The LLVM tests build options shared by all platforms.
///
/// Toggles the LLVM regression and unit tests (`-DLLVM_{BUILD,INCLUDE}_TESTS`).
/// `--enable-tests` implies `--enable-utils` (see `crate::llvm::build`);
/// callers that only need `FileCheck`/`llvm-lit` should prefer
/// `--enable-utils` alone.
///
pub fn shared_build_opts_tests(enabled: bool) -> Vec<String> {
    vec![
        format!(
            "-DLLVM_BUILD_TESTS='{}'",
            if enabled { "On" } else { "Off" },
        ),
        format!(
            "-DLLVM_INCLUDE_TESTS='{}'",
            if enabled { "On" } else { "Off" },
        ),
    ]
}

///
/// The code coverage build options shared by all platforms.
///
pub fn shared_build_opts_coverage(enabled: bool) -> Vec<String> {
    vec![format!(
        "-DLLVM_BUILD_INSTRUMENTED_COVERAGE='{}'",
        if enabled { "On" } else { "Off" },
    )]
}

///
/// Configure an `install-distribution` that ships only what solx needs,
/// skipping the ~200 LLVM tool binaries (`opt`, `llc`, ...).
///
/// solx never invokes any LLVM tool at runtime — it consumes LLVM as a
/// library via inkwell — so linking those tools is pure wall-clock waste in
/// CI. `LLVM_BUILD_TOOLS=Off` marks every tool `EXCLUDE_FROM_ALL`;
/// `LLVM_INCLUDE_TOOLS=On` keeps `tools/` in the configure pass so the
/// umbrella targets (`llvm-libraries`, `lld-libraries`, etc.) stay defined.
/// `LLVM_DISTRIBUTION_COMPONENTS` whitelists what `install-distribution`
/// actually copies into the install prefix — see #364.
///
/// `llvm-config` is deliberately **not** in the distribution list: with
/// `LLVM_BUILD_TOOLS=Off`, cmake's `llvm_add_tool` skips creation of the
/// `install-llvm-config` target, so referencing it here errors at configure
/// time. `llvm-sys` still needs `llvm-config` at Rust build time —
/// `build_and_install_llvm_config` builds it directly via `ninja
/// llvm-config` and copies the binary into the install prefix afterwards.
/// `lld-*` is always included because `shared_build_opts_projects` always
/// enables `lld`. `mlir-*` is included only when `enable_mlir` is true.
///
/// When `enable_utils` is true, `FileCheck` is added to the whitelist —
/// `solx-mlir/tests/lit/*.sol` uses it in RUN lines, so it must land in the
/// install prefix's `bin/`. LLVM's `add_llvm_utility` registers each utility
/// under its own install component (named after the target). The component
/// only exists when `LLVM_BUILD_UTILS=On` (set by `shared_build_opts_utils`
/// when `--enable-utils` is passed); listing it without that flag errors at
/// configure time.
///
/// `llvm-lit` is **not** listed here despite being needed by the slang-tests
/// workflow runner: upstream LLVM has no install rule for `llvm-lit` (it's
/// only `configure_file`'d into the build dir), so referencing it as a
/// distribution component errors at configure time. How `target-final/bin/
/// llvm-lit` reaches the install prefix today is independent of this PR.
///
pub fn build_opts_distribution(enable_mlir: bool, enable_utils: bool) -> Vec<String> {
    let mut components = vec![
        "llvm-libraries",
        "llvm-headers",
        "cmake-exports",
        "lld-libraries",
        "lld-headers",
        "lld-cmake-exports",
    ];
    if enable_mlir {
        components.extend(["mlir-libraries", "mlir-headers", "mlir-cmake-exports"]);
    }
    if enable_utils {
        components.push("FileCheck");
    }
    vec![
        "-DLLVM_BUILD_TOOLS='Off'".to_owned(),
        "-DLLVM_INCLUDE_TOOLS='On'".to_owned(),
        format!("-DLLVM_DISTRIBUTION_COMPONENTS='{}'", components.join(";")),
    ]
}

///
/// Build `llvm-config` directly (it has no install target under
/// `LLVM_BUILD_TOOLS=Off`) and copy it into the install prefix's `bin/`.
///
/// `llvm-sys` invokes `llvm-config` at Rust build time to discover libs and
/// flags. Both ninja args and the destination path use forward-slash form on
/// Windows; callers are expected to pass already-normalized paths (existing
/// Windows code uses `path_windows_to_unix` for this).
///
/// `overwrite: true` lets a second `solx-dev llvm build` against the same
/// target prefix succeed; `fs_extra::file::copy`'s default would error on
/// `AlreadyExists` for the second run.
///
pub fn build_and_install_llvm_config(build_dir: &Path, target_dir: &Path) -> anyhow::Result<()> {
    let exe_suffix = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };
    let bin_name = format!("llvm-config{exe_suffix}");
    let build_dir_str = build_dir.to_string_lossy();

    crate::utils::command(
        Command::new("ninja")
            .arg("-C")
            .arg(&*build_dir_str)
            .arg("llvm-config"),
        "Building llvm-config",
    )?;

    let source = build_dir.join("bin").join(&bin_name);
    let dest_dir = target_dir.join("bin");
    std::fs::create_dir_all(&dest_dir)?;
    fs_extra::file::copy(
        &source,
        dest_dir.join(&bin_name),
        &fs_extra::file::CopyOptions {
            overwrite: true,
            ..Default::default()
        },
    )?;
    Ok(())
}

///
/// Ignore duplicate libraries warnings for MacOS with XCode>=15.
///
pub fn macos_build_opts_ignore_dupicate_libs_warnings() -> Vec<String> {
    let xcode_version =
        crate::utils::get_xcode_version().unwrap_or(crate::utils::XCODE_MIN_VERSION);
    if xcode_version >= crate::utils::XCODE_VERSION_15 {
        vec![
            "-DCMAKE_EXE_LINKER_FLAGS='-Wl,-no_warn_duplicate_libraries'".to_owned(),
            "-DCMAKE_SHARED_LINKER_FLAGS='-Wl,-no_warn_duplicate_libraries'".to_owned(),
        ]
    } else {
        vec![]
    }
}
