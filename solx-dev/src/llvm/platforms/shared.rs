//!
//! The shared options for building various platforms.
//!

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
/// The LLVM tests build options shared by all platforms.
///
pub fn shared_build_opts_tests(enabled: bool) -> Vec<String> {
    vec![
        format!(
            "-DLLVM_BUILD_UTILS='{}'",
            if enabled { "On" } else { "Off" },
        ),
        format!(
            "-DLLVM_BUILD_TESTS='{}'",
            if enabled { "On" } else { "Off" },
        ),
        format!(
            "-DLLVM_INCLUDE_UTILS='{}'",
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
/// Windows-only: configure an `install-distribution` that ships only what solx
/// needs, skipping the ~200 LLVM tool binaries (`opt.exe`, `llc.exe`, ...).
///
/// `lld-link` dominates LLVM build wall-clock on the hosted Windows runner; solx
/// never invokes any LLVM tool at runtime (it consumes LLVM as a library via
/// inkwell), so linking them is pure waste. `LLVM_BUILD_TOOLS=Off` marks every
/// tool `EXCLUDE_FROM_ALL`; `LLVM_INCLUDE_TOOLS=On` keeps `tools/` in the
/// configure pass so the umbrella targets (`llvm-libraries` etc.) are defined.
/// `LLVM_DISTRIBUTION_COMPONENTS` whitelists the install set, which
/// `install-distribution` then honours — see #364.
///
/// `llvm-config` is deliberately **not** in the distribution list: with
/// `LLVM_BUILD_TOOLS=Off`, `llvm_add_tool` never creates the `install-llvm-config`
/// cmake target (both `install(TARGETS)` and `add_llvm_install_targets` are
/// gated on `LLVM_BUILD_TOOLS`), so referencing it here errors at configure
/// time. `llvm-sys` still needs `llvm-config` at Rust build time — the Windows
/// builder builds it via direct `ninja llvm-config` and copies the binary into
/// the install prefix after `install-distribution` completes.
/// `lld-*` is always included because `shared_build_opts_projects` always
/// enables `lld`. `mlir-*` is included only when `enable_mlir` is true.
///
pub fn windows_build_opts_distribution(enable_mlir: bool) -> Vec<String> {
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
    vec![
        "-DLLVM_BUILD_TOOLS='Off'".to_owned(),
        "-DLLVM_INCLUDE_TOOLS='On'".to_owned(),
        format!("-DLLVM_DISTRIBUTION_COMPONENTS='{}'", components.join(";")),
    ]
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
