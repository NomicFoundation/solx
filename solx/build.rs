//!
//! The build script for native libraries used by `solx`.
//!

fn main() {
    #[cfg(feature = "solc")]
    link_solc_libs();

    #[cfg(all(feature = "mlir", feature = "solc"))]
    link_mlir_libs();
}

/// Link solc and Boost libraries from SOLC_PREFIX / BOOST_PREFIX.
#[cfg(feature = "solc")]
fn link_solc_libs() {
    println!("cargo:rerun-if-env-changed=BOOST_PREFIX");
    if let Ok(path) = std::env::var("BOOST_PREFIX") {
        println!("cargo:rerun-if-changed={path}");
    }

    println!("cargo:rerun-if-env-changed=SOLC_PREFIX");
    if let Ok(path) = std::env::var("SOLC_PREFIX") {
        println!("cargo:rerun-if-changed={path}");
    }

    println!("cargo:rustc-link-search=native={}", env!("BOOST_PREFIX"));
    for directory in [
        "libsolc",
        "libsolidity",
        "libsolutil",
        "liblangutil",
        "libevmasm",
        "libyul",
        "libsmtutil",
    ] {
        println!(
            "cargo:rustc-link-search=native={}/{directory}",
            env!("SOLC_PREFIX"),
        );
    }

    for library in ["boost_filesystem", "boost_system", "boost_program_options"] {
        println!("cargo:rustc-link-lib=static={library}");
    }
    for library in [
        "solc", "solidity", "solutil", "langutil", "evmasm", "yul", "smtutil",
    ] {
        println!("cargo:rustc-link-lib=static={library}");
    }
}

/// Link MLIR libraries for the C++ MLIR path (Config #4: solc+mlir).
/// All MLIR libs (core + custom dialects) come from LLVM_SYS_211_PREFIX.
/// Only active when both `mlir` and `solc` features are enabled.
/// When slang is active instead, mlir-sys (via melior in solx-mlir) handles linking.
#[cfg(all(feature = "mlir", feature = "solc"))]
fn link_mlir_libs() {
    use std::path::PathBuf;

    println!("cargo:rerun-if-env-changed=LLVM_SYS_211_PREFIX");
    if let Ok(path) = std::env::var("LLVM_SYS_211_PREFIX") {
        println!("cargo:rerun-if-changed={path}");
    }

    let llvm_lib_path = PathBuf::from(env!("LLVM_SYS_211_PREFIX")).join("lib");
    println!("cargo:rustc-link-search=native={}", llvm_lib_path.display());

    for library in [
        // Custom solx dialects (built in solx-llvm)
        "MLIRSolDialect",
        "MLIRYulDialect",
        // Core dialects
        "MLIRFuncDialect",
        "MLIRSCFDialect",
        "MLIRArithDialect",
        "MLIRLLVMDialect",
        "MLIRControlFlowDialect",
        "MLIRPDLDialect",
        "MLIRPDLInterpDialect",
        "MLIRUBDialect",
        "MLIRTensorDialect",
        "MLIRDLTIDialect",
        // Conversions
        "MLIRFuncToLLVM",
        "MLIRSCFToControlFlow",
        "MLIRControlFlowToLLVM",
        "MLIRArithAttrToLLVMConversion",
        "MLIRArithToLLVM",
        "MLIRLLVMCommonConversion",
        "MLIRPDLToPDLInterp",
        // Translations
        "MLIRBuiltinToLLVMIRTranslation",
        "MLIRLLVMToLLVMIRTranslation",
        "MLIRTargetLLVMIRExport",
        // Interfaces
        "MLIRCallInterfaces",
        "MLIRControlFlowInterfaces",
        "MLIRInferIntRangeInterface",
        "MLIRInferIntRangeCommon",
        "MLIRMemorySlotInterfaces",
        "MLIRDataLayoutInterfaces",
        "MLIRSideEffectInterfaces",
        "MLIRCastInterfaces",
        "MLIRLoopLikeInterface",
        "MLIRFunctionInterfaces",
        "MLIRDestinationStyleOpInterface",
        "MLIRViewLikeInterface",
        "MLIRInferTypeOpInterface",
        "MLIRParallelCombiningOpInterface",
        // Utils and transforms
        "MLIRSCFUtils",
        "MLIRSCFTransforms",
        "MLIRDialectUtils",
        "MLIRArithUtils",
        "MLIRLLVMIRTransforms",
        // Core
        "MLIRSupport",
        "MLIRPass",
        "MLIRTransforms",
        "MLIRTransformUtils",
        "MLIRRewrite",
        "MLIRRewritePDL",
        "MLIRAnalysis",
        "MLIRParser",
        "MLIRIR",
        "MLIRDialect",
    ] {
        println!("cargo:rustc-link-lib=static={library}");
    }
}
