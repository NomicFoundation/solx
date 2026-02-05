//!
//! The shared options for building solc on various platforms.
//!

use std::path::Path;

use crate::build_type::BuildType;

///
/// Common cmake options for building solc.
///
pub fn shared_cmake_args(build_type: BuildType, pedantic: bool, tests: bool) -> Vec<String> {
    vec![
        format!("-DCMAKE_BUILD_TYPE={build_type}"),
        format!("-DPEDANTIC={}", if pedantic { "ON" } else { "OFF" }),
        format!("-DTESTS={}", if tests { "ON" } else { "OFF" }),
        "-DUSE_Z3=OFF".to_owned(),
        "-DUSE_CVC4=OFF".to_owned(),
        "-DSOLC_LINK_STATIC=1".to_owned(),
        "-DSTATIC_LINKING=1".to_owned(),
    ]
}

///
/// CXX flags for building solc.
///
pub fn shared_cxx_flags() -> String {
    "-DJSON_USE_INT64_DOUBLE_CONVERSION -DBOOST_NO_CXX98_FUNCTION_BASE -D_LIBCPP_ENABLE_CXX17_REMOVED_UNARY_BINARY_FUNCTION".to_owned()
}

///
/// Boost cmake arguments.
///
/// Note: The boost paths should be absolute (canonicalized in mod.rs).
/// The boost_root is constructed from the lib_dir since the cmake config
/// directory (lib/cmake/Boost-X.Y.Z) may not exist.
///
pub fn boost_cmake_args(
    boost_version: &str,
    boost_lib_dir: &Path,
    boost_include_dir: &Path,
) -> Vec<String> {
    // Construct boost_root from the lib_dir
    // The cmake config dir may not exist, but CMake will still use the other paths
    let boost_root = boost_lib_dir
        .join("cmake")
        .join(format!("Boost-{boost_version}"));

    vec![
        "-DBoost_NO_BOOST_CMAKE=1".to_owned(),
        "-DBoost_NO_SYSTEM_PATHS=1".to_owned(),
        format!("-DBOOST_ROOT={}", boost_root.display()),
        format!("-DBoost_DIR={}", boost_root.display()),
        format!("-DBOOST_LIBRARYDIR={}", boost_lib_dir.display()),
        "-DBoost_USE_STATIC_RUNTIME=1".to_owned(),
        "-DBOOST_USE_STATIC_LIBS=1".to_owned(),
        format!("-DBOOST_INCLUDEDIR={}", boost_include_dir.display()),
        "-DBoost_DEBUG=1".to_owned(),
    ]
}

///
/// MLIR cmake arguments (when MLIR is enabled).
///
pub fn mlir_cmake_args(llvm_build_dir: &Path) -> Vec<String> {
    let mlir_dir = llvm_build_dir.join("lib/cmake/mlir");
    let lld_dir = llvm_build_dir.join("lib/cmake/lld");

    let mut args = Vec::new();
    if mlir_dir.exists() {
        args.push(format!("-DMLIR_DIR={}", mlir_dir.display()));
    }
    if lld_dir.exists() {
        args.push(format!("-DLLD_DIR={}", lld_dir.display()));
    }
    args
}
