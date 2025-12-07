//!
//! The shared options for building various platforms.
//!

use crate::llvm::ccache_variant::CcacheVariant;
use crate::llvm::platforms::Platform;
use crate::llvm::sanitizer::Sanitizer;

/// The build options shared by all platforms.
pub const SHARED_BUILD_OPTS: [&str; 22] = [
    "-DPACKAGE_VENDOR='Matter Labs'",
    "-DCMAKE_BUILD_WITH_INSTALL_RPATH=1",
    "-DLLVM_BUILD_DOCS='Off'",
    "-DLLVM_BUILD_RUNTIME='Off'",
    "-DLLVM_BUILD_RUNTIMES='Off'",
    "-DLLVM_INCLUDE_DOCS='Off'",
    "-DLLVM_INCLUDE_BENCHMARKS='Off'",
    "-DLLVM_INCLUDE_EXAMPLES='Off'",
    "-DLLVM_INCLUDE_RUNTIMES='Off'",
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
    "-DBUG_REPORT_URL='https://github.com/matter-labs/era-compiler-llvm/issues/'",
];

///
/// The shared build options to treat warnings as errors.
///
/// Disabled on Windows due to the following upstream issue with MSYS2 with mingw-w64:
/// ProgramTest.cpp:23:15: error: '__p__environ' redeclared without 'dllimport' attribute
///
pub fn shared_build_opts_werror() -> Vec<String> {
    vec![format!(
        "-DLLVM_ENABLE_WERROR='{}'",
        if cfg!(target_os = "windows") {
            "Off"
        } else {
            "On"
        },
    )]
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
/// The build options to build with RTTI support.
///
pub fn shared_build_opts_rtti(enabled: bool) -> Vec<String> {
    vec![format!(
        "-DLLVM_ENABLE_RTTI='{}'",
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
            [Platform::EVM, Platform::EraVM]
                .into_iter()
                .map(|platform| platform.to_string())
                .collect::<Vec<String>>()
                .join(";")
        ),
        format!("-DLLVM_DEFAULT_TARGET_TRIPLE='{}'", Platform::EVM),
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
/// Use of compiler cache (ccache) to speed up the build process.
///
pub fn shared_build_opts_ccache(ccache_variant: Option<CcacheVariant>) -> Vec<String> {
    match ccache_variant {
        Some(ccache_variant) => vec![
            format!(
                "-DCMAKE_C_COMPILER_LAUNCHER='{}'",
                ccache_variant.to_string()
            ),
            format!(
                "-DCMAKE_CXX_COMPILER_LAUNCHER='{}'",
                ccache_variant.to_string()
            ),
        ],
        None => vec![],
    }
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
